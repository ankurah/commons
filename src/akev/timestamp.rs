//! An event's timestamp in the one form a file writes it, `YYYY-MM-DDTHH:MM:SS.mmmZ`: UTC,
//! with exactly three decimals, so that a timestamp has one spelling.

use std::{error::Error, fmt};

const MS_PER_DAY: u64 = 86_400_000;

/// Days from 0000-03-01 to 1970-01-01 in the proleptic Gregorian calendar.
const EPOCH_DAYS: u64 = 719_468;

/// Days in one 400-year cycle of the Gregorian calendar.
const CYCLE_DAYS: u64 = 146_097;

/// Why a timestamp does not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampError {
    /// Not `YYYY-MM-DDTHH:MM:SS.mmmZ`.
    Form,
    /// In the form, but no moment from 1970 on: a month, day, hour, minute or second out of
    /// its range, or a year before 1970.
    NoSuchMoment,
}

impl fmt::Display for TimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimestampError::Form => f.write_str("a timestamp is written YYYY-MM-DDTHH:MM:SS.mmmZ"),
            TimestampError::NoSuchMoment => f.write_str("names no moment from 1970 on"),
        }
    }
}

impl Error for TimestampError {}

/// The last Unix millisecond the form writes, at the end of the year 9999.
pub const LAST: u64 = 253_402_300_799_999;

/// `ms`, Unix milliseconds, in the form; none after `LAST`.
pub fn format(ms: u64) -> Option<String> {
    if ms > LAST {
        return None;
    }
    let (year, month, day) = civil(ms / MS_PER_DAY);
    let of_day = ms % MS_PER_DAY;
    let (hour, minute, second, milli) = (of_day / 3_600_000, of_day / 60_000 % 60, of_day / 1000 % 60, of_day % 1000);
    Some(format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}Z"))
}

/// The Unix milliseconds `text` names, in exactly the form.
pub fn parse(text: &str) -> Result<u64, TimestampError> {
    let bytes = text.as_bytes();
    let form = b"dddd-dd-ddTdd:dd:dd.dddZ";
    let in_form = bytes.len() == form.len()
        && bytes.iter().zip(form).all(|(byte, expected)| if *expected == b'd' { byte.is_ascii_digit() } else { byte == expected });
    if !in_form {
        return Err(TimestampError::Form);
    }
    let number = |range: std::ops::Range<usize>| text[range].parse::<u64>().map_err(|_| TimestampError::Form);
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second, milli) = (number(11..13)?, number(14..16)?, number(17..19)?, number(20..23)?);
    if year < 1970 || !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) || hour > 23 || minute > 59 || second > 59
    {
        return Err(TimestampError::NoSuchMoment);
    }
    Ok(days(year, month, day) * MS_PER_DAY + ((hour * 60 + minute) * 60 + second) * 1000 + milli)
}

fn days_in_month(year: u64, month: u64) -> u64 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a date from then on, counting years from March so that a leap
/// day ends its year (Howard Hinnant's `days_from_civil`).
fn days(year: u64, month: u64, day: u64) -> u64 {
    let year = if month <= 2 { year - 1 } else { year };
    let (cycle, of_cycle) = (year / 400, year % 400);
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let of_cycle = of_cycle * 365 + of_cycle / 4 - of_cycle / 100 + of_year;
    cycle * CYCLE_DAYS + of_cycle - EPOCH_DAYS
}

/// The date `days` after 1970-01-01, the inverse of `days` (Hinnant's `civil_from_days`).
fn civil(days: u64) -> (u64, u64, u64) {
    let days = days + EPOCH_DAYS;
    let (cycle, of_cycle) = (days / CYCLE_DAYS, days % CYCLE_DAYS);
    let year_of_cycle = (of_cycle - of_cycle / 1460 + of_cycle / 36_524 - of_cycle / (CYCLE_DAYS - 1)) / 365;
    let of_year = of_cycle - (365 * year_of_cycle + year_of_cycle / 4 - year_of_cycle / 100);
    let march_month = (5 * of_year + 2) / 153;
    let day = of_year - (153 * march_month + 2) / 5 + 1;
    let month = if march_month < 10 { march_month + 3 } else { march_month - 9 };
    let year = cycle * 400 + year_of_cycle + u64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Moments around the epoch, leap days, the turns of centuries and the last moment the
    /// form writes, each with its one spelling.
    const KNOWN: &[(u64, &str)] = &[
        (0, "1970-01-01T00:00:00.000Z"),
        (946_684_799_999, "1999-12-31T23:59:59.999Z"),
        (951_868_799_999, "2000-02-29T23:59:59.999Z"),
        (1_709_208_000_000, "2024-02-29T12:00:00.000Z"),
        (1_794_795_263_846, "2026-11-16T02:14:23.846Z"),
        (4_107_542_400_000, "2100-03-01T00:00:00.000Z"),
        (LAST, "9999-12-31T23:59:59.999Z"),
    ];

    #[test]
    fn a_known_moment_writes_its_spelling_and_the_spelling_reads_as_the_moment() {
        for &(ms, text) in KNOWN {
            assert_eq!(format(ms).as_deref(), Some(text));
            assert_eq!(parse(text), Ok(ms), "{text}");
        }
    }

    /// Moments about eleven and a half days apart, a step that is no whole number of seconds,
    /// so that every field of the form takes many values.
    #[test]
    fn every_moment_written_reads_back_as_itself() {
        for ms in (0..=LAST).step_by(1_000_003_331) {
            assert_eq!(parse(&format(ms).unwrap()), Ok(ms));
        }
    }

    #[test]
    fn no_moment_after_the_year_9999_is_written() {
        assert_eq!(format(LAST + 1), None);
    }

    #[test]
    fn any_other_spelling_or_an_impossible_date_is_refused() {
        let other_forms = [
            "2026-11-16T02:14:23.846",
            "2026-11-16T02:14:23.84Z",
            "2026-11-16T02:14:23.8460Z",
            "2026-11-16 02:14:23.846Z",
            "2026-11-16T02:14:23Z",
            "2026-11-16t02:14:23.846z",
            "+026-11-16T02:14:23.846Z",
            "1794795263846",
        ];
        for text in other_forms {
            assert_eq!(parse(text), Err(TimestampError::Form), "{text}");
        }
        let impossible = [
            "2026-02-29T00:00:00.000Z",
            "2100-02-29T00:00:00.000Z",
            "2026-13-01T00:00:00.000Z",
            "2026-04-31T00:00:00.000Z",
            "2026-11-00T00:00:00.000Z",
            "2026-11-16T24:00:00.000Z",
            "2026-11-16T02:60:00.000Z",
            "2026-11-16T02:14:60.000Z",
            "1969-12-31T23:59:59.999Z",
        ];
        for text in impossible {
            assert_eq!(parse(text), Err(TimestampError::NoSuchMoment), "{text}");
        }
    }
}
