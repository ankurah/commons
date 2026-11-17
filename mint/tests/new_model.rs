//! Minting a model declared anew. The tool builds from the declarations alone, before the
//! model has a kept file or an id; it mints the model and gives its struct the id, and, built
//! again, it verifies the mint.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ankurah_commons::akev;

/// A model declared after the mint was kept.
const HUNCH: &str = "use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A model declared for a test.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = \"cognition.hunch\")]
#[model(no_ffi)]
pub struct Hunch {
    #[active_type(LWW)]
    pub words: String,
}
";

/// The workspace: the crate, and the tool beside it.
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_owned()
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy(&path, &target);
        } else {
            fs::copy(&path, &target).unwrap();
        }
    }
}

/// Edits the first occurrence of `from` in the file at `path`.
fn edit(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{} holds no `{from}`", path.display());
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}

/// Builds the tool in the workspace at `root` and runs its `command`, failing with what it
/// printed unless it succeeds.
fn run(root: &Path, command: &str) {
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["run", "--quiet", "--offline", "--locked", "-p", "ankurah-commons-mint", "--", command])
        // A build directory of its own, kept between runs, so that only the copied crates build again.
        .env("CARGO_TARGET_DIR", workspace().join("target").join("new-model"))
        .output()
        .unwrap();
    let printed = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "{command} failed:\n{printed}");
}

#[test]
fn a_model_declared_anew_is_minted_from_its_declaration_alone() {
    let root = tempfile::tempdir().unwrap();
    for file in ["Cargo.toml", "Cargo.lock", "mint/Cargo.toml"] {
        fs::create_dir_all(root.path().join(file).parent().unwrap()).unwrap();
        fs::copy(workspace().join(file), root.path().join(file)).unwrap();
    }
    copy(&workspace().join("src"), &root.path().join("src"));
    copy(&workspace().join("mint").join("src"), &root.path().join("mint").join("src"));

    let src = root.path().join("src");
    let cognition = src.join("cognition");
    fs::write(cognition.join("hunch.rs"), HUNCH).unwrap();
    edit(&cognition.join("mod.rs"), "mod intuition;\n", "mod hunch;\nmod intuition;\n");
    edit(&cognition.join("mod.rs"), "pub use intuition::*;\n", "pub use hunch::*;\npub use intuition::*;\n");
    let listed = "declared::<crate::cognition::Intuition>(),";
    edit(&src.join("namespaces.rs"), listed, &format!("declared::<crate::cognition::Hunch>(),\n            {listed}"));

    run(root.path(), "mint");
    let hunch = cognition.join("hunch.akev");
    let text = fs::read_to_string(&hunch).unwrap();
    let events = akev::read(&[(&hunch, &text)]).unwrap().remove(0);
    assert_eq!(events.len(), 3, "the model, the property `words` and its link:\n{text}");
    assert!(text.contains(r#"lww: { label: "cognition.hunch", name: "Hunch" }, member: system:model"#), "{text}");
    assert!(text.contains(r#"    name: "words""#), "{text}");
    let declared = format!("#[model(label = \"cognition.hunch\")]\n#[model(id = \"{}\")]\n", events[0].entity_id);
    assert!(fs::read_to_string(cognition.join("hunch.rs")).unwrap().contains(&declared));
    for kept in ["intuition.akev", "intention.akev"] {
        assert_eq!(
            fs::read_to_string(cognition.join(kept)).unwrap(),
            fs::read_to_string(workspace().join("src/cognition").join(kept)).unwrap()
        );
    }
    run(root.path(), "verify");
}
