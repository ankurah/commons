//! The Commons' namespaces and the models each declares: the one list of what `mint`
//! registers. A test below compares it with the `#[derive(Model)]` structs in the
//! namespace modules' sources, so that a model added there and not here fails the tests.

use std::{future::Future, pin::Pin};

use ankurah::{
    core::schema::{registration::RegistrationError, ModelStructDescriptor},
    Context, Model, ModelId,
};
use ankurah_commons::{git, health, highlights, identity, intention_graph, messaging, records};

/// A namespace of the Commons: the module named `name`, with `-` written `_`, and the
/// models it declares.
pub struct Namespace {
    pub name: &'static str,
    pub models: &'static [DeclaredModel],
}

/// A model to mint: its declaration, and the call that registers it.
pub struct DeclaredModel {
    /// The declaration `#[derive(Model)]` compiled from the model's struct.
    pub descriptor: fn() -> &'static ModelStructDescriptor,
    /// Registers the model on the context's node, which mints its definitions there,
    /// and returns the model's id.
    pub register: for<'c> fn(&'c Context) -> Registration<'c>,
}

/// The registration of one model, under way.
pub type Registration<'c> = Pin<Box<dyn Future<Output = Result<ModelId, RegistrationError>> + 'c>>;

const fn declared<M: Model + 'static>() -> DeclaredModel {
    DeclaredModel { descriptor: M::descriptor, register: register::<M> }
}

fn register<M: Model + 'static>(context: &Context) -> Registration<'_> {
    Box::pin(context.resolve_model_id::<M>())
}

/// Every namespace that declares models, in the order the workspace lists them, and
/// each one's models in the order its crate declares their modules.
pub const NAMESPACES: &[Namespace] = &[
    Namespace { name: "identity", models: &[declared::<identity::Credential>(), declared::<identity::Principal>()] },
    Namespace {
        name: "records",
        models: &[
            declared::<records::Assignment>(),
            declared::<records::Attribution>(),
            declared::<records::Closure>(),
            declared::<records::Creation>(),
            declared::<records::LastEdit>(),
            declared::<records::Ownership>(),
        ],
    },
    Namespace {
        name: "health",
        models: &[
            declared::<health::ActivityInterval>(),
            declared::<health::Distance>(),
            declared::<health::HeartRate>(),
            declared::<health::HeartRateSummary>(),
            declared::<health::PhysicalActivity>(),
            declared::<health::Workout>(),
        ],
    },
    Namespace { name: "highlights", models: &[declared::<highlights::Fragment>()] },
    Namespace { name: "git", models: &[declared::<git::FileAnchor>(), declared::<git::Repository>(), declared::<git::WorkLocation>()] },
    Namespace {
        name: "intention-graph",
        models: &[
            declared::<intention_graph::Claim>(),
            declared::<intention_graph::Decision>(),
            declared::<intention_graph::Intention>(),
            declared::<intention_graph::Link>(),
            declared::<intention_graph::Reading>(),
            declared::<intention_graph::Scope>(),
            declared::<intention_graph::Submission>(),
            declared::<intention_graph::Task>(),
        ],
    },
    Namespace { name: "messaging", models: &[declared::<messaging::Mention>(), declared::<messaging::Message>()] },
];

/// The guard that keeps `NAMESPACES` complete, so that a model added to a namespace
/// module, or a new namespace module, cannot be forgotten in the list `mint` registers.
/// It relies on one convention: a model is a struct with `#[derive(Model, ..)]` written
/// directly on it, in a file under its namespace's directory in `src/`. A model declared
/// any other way is not seen.
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::{Path, PathBuf},
    };
    use syn::{punctuated::Punctuated, Token};

    /// Model names by namespace.
    type Models = BTreeMap<String, Vec<String>>;

    /// The first way the list and the sources differ.
    #[derive(Debug, PartialEq, Eq)]
    enum Difference {
        /// A namespace that only one of the list and the modules in `src/` has.
        Namespace(String),
        NotListed {
            namespace: String,
            model: String,
        },
        NotDeclared {
            namespace: String,
            model: String,
        },
        Repeated {
            namespace: String,
        },
    }

    fn listed(namespaces: &[Namespace]) -> Models {
        let names = |namespace: &Namespace| namespace.models.iter().map(|model| (model.descriptor)().name.to_owned()).collect();
        namespaces.iter().map(|namespace| (namespace.name.to_owned(), names(namespace))).collect()
    }

    /// The models the namespace modules in `src` declare: every directory there, named as
    /// its namespace with `-` written `_`.
    fn declared(src: &Path) -> Models {
        let modules = fs::read_dir(src).unwrap().map(|entry| entry.unwrap().path()).filter(|path| path.is_dir());
        modules
            .map(|module| {
                let mut names = Vec::new();
                collect(&module, &mut names);
                (module.file_name().unwrap().to_str().unwrap().replace('_', "-"), names)
            })
            .collect()
    }

    /// Adds to `names` every struct that a Rust file under `dir` derives `Model` for.
    fn collect(dir: &Path, names: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, names);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let file = syn::parse_file(&fs::read_to_string(&path).unwrap()).unwrap();
                for item in file.items {
                    if let syn::Item::Struct(item) = item {
                        if derives_model(&item.attrs) {
                            names.push(item.ident.to_string());
                        }
                    }
                }
            }
        }
    }

    fn derives_model(attributes: &[syn::Attribute]) -> bool {
        attributes.iter().filter(|attribute| attribute.path().is_ident("derive")).any(|attribute| {
            let derived = attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated).unwrap();
            derived.iter().any(|path| path.segments.last().is_some_and(|segment| segment.ident == "Model"))
        })
    }

    fn difference(listed: &Models, declared: &Models) -> Option<Difference> {
        for namespace in declared.keys().chain(listed.keys()) {
            let (Some(list), Some(models)) = (listed.get(namespace), declared.get(namespace)) else {
                return Some(Difference::Namespace(namespace.clone()));
            };
            if let Some(model) = models.iter().find(|model| !list.contains(model)) {
                return Some(Difference::NotListed { namespace: namespace.clone(), model: model.clone() });
            }
            if let Some(model) = list.iter().find(|model| !models.contains(model)) {
                return Some(Difference::NotDeclared { namespace: namespace.clone(), model: model.clone() });
            }
            if list.len() != models.len() {
                return Some(Difference::Repeated { namespace: namespace.clone() });
            }
        }
        None
    }

    fn sources() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("the mint crate sits in the workspace").join("src")
    }

    #[test]
    fn the_list_names_every_model_the_namespace_modules_declare_and_each_label_once() {
        assert_eq!(difference(&listed(NAMESPACES), &declared(&sources())), None);
        let labels: BTreeSet<&str> =
            NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| (model.descriptor)().label).collect();
        assert_eq!(labels.len(), NAMESPACES.iter().map(|namespace| namespace.models.len()).sum::<usize>(), "a label is listed twice");
    }

    #[test]
    fn a_model_removed_from_the_list_is_named() {
        let mut list = listed(NAMESPACES);
        let model = list.get_mut("identity").unwrap().remove(0);
        assert_eq!(difference(&list, &declared(&sources())), Some(Difference::NotListed { namespace: "identity".to_owned(), model }));
    }

    #[test]
    fn an_unlisted_model_or_namespace_module_is_named() {
        let src = tempfile::tempdir().unwrap();
        fs::write(src.path().join("lib.rs"), "").unwrap();
        let alpha = "#[derive(Debug, Model)]\npub struct Listed {}\n#[derive(ankurah::Model)]\npub struct Unlisted {}\n";
        for (module, source) in [("alpha", alpha), ("beta", "")] {
            fs::create_dir_all(src.path().join(module)).unwrap();
            fs::write(src.path().join(module).join("mod.rs"), source).unwrap();
        }
        let list = |entries: &[(&str, &[&str])]| -> Models {
            entries
                .iter()
                .map(|(namespace, models)| (namespace.to_string(), models.iter().map(|model| model.to_string()).collect()))
                .collect()
        };
        let declared = declared(src.path());
        let unlisted = Difference::NotListed { namespace: "alpha".to_owned(), model: "Unlisted".to_owned() };
        assert_eq!(difference(&list(&[("alpha", &["Listed"]), ("beta", &[])]), &declared), Some(unlisted));
        let module = Difference::Namespace("beta".to_owned());
        assert_eq!(difference(&list(&[("alpha", &["Listed", "Unlisted"])]), &declared), Some(module));
    }
}
