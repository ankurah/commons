//! Splits a file into tokens, each with the line it stands on. White space and comments
//! go; what stays of them is the line break between two lines of a block and the blank
//! line that ends one.

use super::read_error::{At, Problem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token<'t> {
    /// `[`, `]`, `{`, `}` or `,`.
    Mark(char),
    /// A run of word characters: a key with its colon, a value, or both.
    Word(&'t str),
    /// A quoted string's text, its escapes undone.
    Text(String),
    /// A line break between two lines of a block.
    LineBreak,
    /// A line that holds nothing, not even a comment, which ends a block.
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Lexed<'t> {
    pub token: Token<'t>,
    /// The line, counted from one; for a line break or a blank line, the line it ends.
    pub line: usize,
}

/// The tokens of `text`.
pub(super) fn tokens(text: &str) -> Result<Vec<Lexed<'_>>, At> {
    let mut tokens = Vec::new();
    let mut line = 1;
    // Whether the current line holds a token or a comment, and so is not blank.
    let mut held = false;
    let mut at = 0;
    while let Some(character) = text[at..].chars().next() {
        let rest = &text[at..];
        match character {
            '\n' => {
                tokens.push(Lexed { token: if held { Token::LineBreak } else { Token::Blank }, line });
                line += 1;
                held = false;
                at += 1;
            }
            ' ' | '\t' | '\r' => at += 1,
            _ if rest.starts_with("//") => {
                held = true;
                at += rest.find('\n').unwrap_or(rest.len());
            }
            _ if rest.starts_with("/*") => {
                held = true;
                let length = rest[2..].find("*/").ok_or(At { line, problem: Problem::UnclosedComment })? + 4;
                // A comment stands for the white space it spans, line breaks and all; every
                // line it touches holds it.
                for _ in rest[..length].matches('\n') {
                    tokens.push(Lexed { token: Token::LineBreak, line });
                    line += 1;
                }
                at += length;
            }
            '[' | ']' | '{' | '}' | ',' => {
                held = true;
                tokens.push(Lexed { token: Token::Mark(character), line });
                at += 1;
            }
            '"' => {
                held = true;
                let (text, length) = quoted(&rest[1..]).map_err(|problem| At { line, problem })?;
                tokens.push(Lexed { token: Token::Text(text), line });
                at += 1 + length;
            }
            _ if is_word(character) => {
                held = true;
                let length = word(rest);
                tokens.push(Lexed { token: Token::Word(&rest[..length]), line });
                at += length;
            }
            _ => return Err(At { line, problem: Problem::UnexpectedCharacter(character) }),
        }
    }
    Ok(tokens)
}

/// The characters of a word: those of base64 ids and bytes, built-in words, model ids such
/// as `system:model`, timestamps, names and the paths of files.
fn is_word(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ':' | '$' | '@' | '/')
}

/// The length of the word that starts `text`, which ends where a comment starts.
fn word(text: &str) -> usize {
    let mut length = 0;
    for character in text.chars() {
        let rest = &text[length..];
        if !is_word(character) || rest.starts_with("//") || rest.starts_with("/*") {
            break;
        }
        length += character.len_utf8();
    }
    length
}

/// The text of the string literal whose opening quote comes just before `text`, and the
/// length of the literal after that quote, its closing quote included.
fn quoted(text: &str) -> Result<(String, usize), Problem> {
    let mut unquoted = String::new();
    let mut characters = text.char_indices();
    while let Some((index, character)) = characters.next() {
        match character {
            '"' => return Ok((unquoted, index + 1)),
            '\n' => return Err(Problem::LineBreakInText),
            '\\' => match characters.next() {
                Some((_, '"')) => unquoted.push('"'),
                Some((_, '\\')) => unquoted.push('\\'),
                Some((_, 'n')) => unquoted.push('\n'),
                Some((_, '\n')) | None => return Err(Problem::UnclosedText),
                Some((_, other)) => return Err(Problem::UnknownEscape(other)),
            },
            _ => unquoted.push(character),
        }
    }
    Err(Problem::UnclosedText)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    fn lexed(text: &str) -> Vec<(Token<'_>, usize)> {
        tokens(text).unwrap().into_iter().map(|lexed| (lexed.token, lexed.line)).collect()
    }

    fn refused(text: &str) -> At {
        tokens(text).unwrap_err()
    }

    #[test]
    fn a_line_that_holds_only_a_comment_breaks_the_line_and_an_empty_one_is_blank() {
        let text = "[id] // header\n// note\n  \nkey: \"a \\\"b\\\" \\\\ c\\nd\", x:y\n";
        assert_eq!(
            lexed(text),
            [
                (Mark('['), 1),
                (Word("id"), 1),
                (Mark(']'), 1),
                (LineBreak, 1),
                (LineBreak, 2),
                (Blank, 3),
                (Word("key:"), 4),
                (Text("a \"b\" \\ c\nd".to_owned()), 4),
                (Mark(','), 4),
                (Word("x:y"), 4),
                (LineBreak, 4)
            ]
        );
    }

    #[test]
    fn a_block_comment_stands_for_the_lines_it_spans_and_none_of_them_is_blank() {
        assert_eq!(
            lexed("a /* one\n\ntwo */ b\n\n"),
            [(Word("a"), 1), (LineBreak, 1), (LineBreak, 2), (Word("b"), 3), (LineBreak, 3), (Blank, 4)]
        );
    }

    #[test]
    fn a_word_ends_where_a_comment_starts() {
        assert_eq!(lexed("$x@dir/file.akev//c\nb/*c*/"), [(Word("$x@dir/file.akev"), 1), (LineBreak, 1), (Word("b"), 2)]);
    }

    #[test]
    fn an_unclosed_comment_or_string_a_line_break_in_a_string_or_an_unknown_escape_is_refused_naming_the_line() {
        assert!(matches!(refused("a\n/* b"), At { line: 2, problem: Problem::UnclosedComment }));
        assert!(matches!(refused("a\n\"b"), At { line: 2, problem: Problem::UnclosedText }));
        assert!(matches!(refused("\"b\nc\""), At { line: 1, problem: Problem::LineBreakInText }));
        assert!(matches!(refused("\"\\t\""), At { line: 1, problem: Problem::UnknownEscape('t') }));
        assert!(matches!(refused("a\nb; c"), At { line: 2, problem: Problem::UnexpectedCharacter(';') }));
    }
}
