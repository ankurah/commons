//! The namespaces this build enables, with the models each declares and the kinds of edge
//! `graph` mints: what the mint tool mints, with every namespace enabled, and what a seeded
//! node binds. The text of the kept mint is compiled in by `seed` instead, so that a model
//! can be listed, and minted, before it has a file. A test below compares the list with the
//! `#[derive(Model)]` structs in the namespace modules' sources, so that a model added there
//! and not here fails.

use ankurah::{core::schema::ModelStructDescriptor, Model};

/// A model's declaration, which `#[derive(Model)]` compiles from its struct.
pub type Declaration = fn() -> &'static ModelStructDescriptor;

/// A namespace of the Commons: the module named `name`, the declarations of its models, and
/// the kinds of edge minted with it.
pub struct Namespace {
    pub name: &'static str,
    pub models: &'static [Declaration],
    /// Only `graph` mints any.
    pub edge_kinds: Option<MintedEdgeKinds>,
}

/// The kinds of edge a namespace mints, each a record of one of its models.
pub struct MintedEdgeKinds {
    /// The model whose records they are.
    pub model: Declaration,
    /// Each kind's name and description.
    pub kinds: &'static [(&'static str, &'static str)],
}

// Unused by a build that enables no namespace.
#[allow(dead_code)]
pub(crate) const fn declared<M: Model>() -> Declaration {
    M::descriptor
}

/// Every namespace this build enables, in the order the crate lists them, and each one's
/// models in the order its module declares them.
pub const NAMESPACES: &[Namespace] = &[
    #[cfg(feature = "identity")]
    Namespace {
        name: "identity",
        models: &[declared::<crate::identity::IdentityBinding>(), declared::<crate::identity::Principal>()],
        edge_kinds: None,
    },
    #[cfg(feature = "records")]
    Namespace {
        name: "records",
        models: &[
            declared::<crate::records::Deferred>(),
            declared::<crate::records::FractionalPosition>(),
            declared::<crate::records::Acceptance>(),
            declared::<crate::records::Checked>(),
            declared::<crate::records::Closed>(),
            declared::<crate::records::Created>(),
            declared::<crate::records::Edited>(),
            declared::<crate::records::Note>(),
            declared::<crate::records::Attribution>(),
            declared::<crate::records::Via>(),
            declared::<crate::records::Assignment>(),
            declared::<crate::records::Ownership>(),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "health")]
    Namespace {
        name: "health",
        models: &[
            declared::<crate::health::ActivityInterval>(),
            declared::<crate::health::Distance>(),
            declared::<crate::health::HeartRate>(),
            declared::<crate::health::HeartRateSummary>(),
            declared::<crate::health::PhysicalActivity>(),
            declared::<crate::health::Workout>(),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "text")]
    Namespace { name: "text", models: &[declared::<crate::text::Fragment>(), declared::<crate::text::Quotation>()], edge_kinds: None },
    #[cfg(feature = "git")]
    Namespace {
        name: "git",
        models: &[
            declared::<crate::git::CheckoutState>(),
            declared::<crate::git::FileAnchor>(),
            declared::<crate::git::Repository>(),
            declared::<crate::git::WorkLocation>(),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "graph")]
    Namespace {
        name: "graph",
        models: &[declared::<crate::graph::DynEdge>(), declared::<crate::graph::EdgeKind>()],
        edge_kinds: Some(MintedEdgeKinds { model: declared::<crate::graph::EdgeKind>(), kinds: crate::graph::EDGE_KINDS }),
    },
    #[cfg(feature = "cognition")]
    Namespace {
        name: "cognition",
        models: &[
            declared::<crate::cognition::Claim>(),
            declared::<crate::cognition::Decision>(),
            declared::<crate::cognition::Expectation>(),
            declared::<crate::cognition::Intention>(),
            declared::<crate::cognition::Intuition>(),
            declared::<crate::cognition::Perspective>(),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "work")]
    Namespace {
        name: "work",
        models: &[declared::<crate::work::Scope>(), declared::<crate::work::Submission>(), declared::<crate::work::Task>()],
        edge_kinds: None,
    },
    #[cfg(feature = "messaging")]
    Namespace {
        name: "messaging",
        models: &[declared::<crate::messaging::Mention>(), declared::<crate::messaging::Message>()],
        edge_kinds: None,
    },
];

/// The guard that keeps `NAMESPACES` complete, so that a model added to a namespace module,
/// or a new namespace module, cannot be forgotten in the list. It needs every namespace
/// enabled. It relies on one convention: a model is a struct with `#[derive(Model, ..)]`
/// written directly on it, in a file under its namespace's directory in `src/`. A model
/// declared any other way is not seen.
#[cfg(all(test, feature = "full"))]
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
        let names = |namespace: &Namespace| namespace.models.iter().map(|model| model().name.to_owned()).collect();
        namespaces.iter().map(|namespace| (namespace.name.to_owned(), names(namespace))).collect()
    }

    /// The models the namespace modules in `src` declare: every directory there, named as
    /// its namespace.
    fn declared(src: &Path) -> Models {
        let modules = fs::read_dir(src).unwrap().map(|entry| entry.unwrap().path()).filter(|path| path.is_dir());
        modules
            .map(|module| {
                let mut names = Vec::new();
                collect(&module, &mut names);
                (module.file_name().unwrap().to_str().unwrap().to_owned(), names)
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    #[test]
    fn the_list_names_every_model_the_namespace_modules_declare_and_each_label_once() {
        assert_eq!(difference(&listed(NAMESPACES), &declared(&sources())), None);
        let labels: BTreeSet<&str> = NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| model().label).collect();
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
