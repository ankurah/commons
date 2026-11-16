//! The namespaces this build enables, each with the models it declares and the text of
//! their kept mint: what `seed` loads into a node, and, with every namespace enabled, what
//! the mint tool mints. A test below compares the list with the `#[derive(Model)]` structs
//! in the namespace modules' sources, so that a model added there and not here fails.

use ankurah::{core::schema::ModelStructDescriptor, Model};

/// A namespace of the Commons: the module named `name`, the models it declares, and the
/// kinds of edge minted with it.
pub struct Namespace {
    pub name: &'static str,
    pub models: &'static [MintedModel],
    /// Only `graph` mints any.
    pub edge_kinds: Option<MintedEdgeKinds>,
}

/// A model, and the text of its minted definitions: the `.akev` file beside its struct.
pub struct MintedModel {
    /// The declaration `#[derive(Model)]` compiled from the model's struct.
    pub descriptor: fn() -> &'static ModelStructDescriptor,
    pub akev: &'static str,
}

/// The kinds of edge a namespace mints, each a record of one of its models.
pub struct MintedEdgeKinds {
    /// The declaration of the model whose records they are.
    pub model: fn() -> &'static ModelStructDescriptor,
    /// Each kind's name and description.
    pub kinds: &'static [(&'static str, &'static str)],
    /// The text of their minted records: `edge_kinds.akev` in the namespace's directory.
    pub akev: &'static str,
}

// Unused by a build that enables no namespace.
#[allow(dead_code)]
const fn minted<M: Model>(akev: &'static str) -> MintedModel {
    MintedModel { descriptor: M::descriptor, akev }
}

/// Every namespace this build enables, in the order the crate lists them, and each one's
/// models in the order its module declares them.
pub const NAMESPACES: &[Namespace] = &[
    #[cfg(feature = "identity")]
    Namespace {
        name: "identity",
        models: &[
            minted::<crate::identity::IdentityBinding>(include_str!("identity/identity_binding.akev")),
            minted::<crate::identity::Principal>(include_str!("identity/principal.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "records")]
    Namespace {
        name: "records",
        models: &[
            minted::<crate::records::Deferred>(include_str!("records/deferred.akev")),
            minted::<crate::records::FractionalPosition>(include_str!("records/fractional_position.akev")),
            minted::<crate::records::Acceptance>(include_str!("records/lifecycle/acceptance.akev")),
            minted::<crate::records::Checked>(include_str!("records/lifecycle/checked.akev")),
            minted::<crate::records::Closed>(include_str!("records/lifecycle/closed.akev")),
            minted::<crate::records::Created>(include_str!("records/lifecycle/created.akev")),
            minted::<crate::records::Edited>(include_str!("records/lifecycle/edited.akev")),
            minted::<crate::records::Note>(include_str!("records/note.akev")),
            minted::<crate::records::Attribution>(include_str!("records/provenance/attribution.akev")),
            minted::<crate::records::Via>(include_str!("records/provenance/via.akev")),
            minted::<crate::records::Assignment>(include_str!("records/responsibility/assignment.akev")),
            minted::<crate::records::Ownership>(include_str!("records/responsibility/ownership.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "health")]
    Namespace {
        name: "health",
        models: &[
            minted::<crate::health::ActivityInterval>(include_str!("health/activity_interval.akev")),
            minted::<crate::health::Distance>(include_str!("health/distance.akev")),
            minted::<crate::health::HeartRate>(include_str!("health/heart_rate.akev")),
            minted::<crate::health::HeartRateSummary>(include_str!("health/heart_rate_summary.akev")),
            minted::<crate::health::PhysicalActivity>(include_str!("health/physical_activity.akev")),
            minted::<crate::health::Workout>(include_str!("health/workout.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "text")]
    Namespace {
        name: "text",
        models: &[
            minted::<crate::text::Fragment>(include_str!("text/fragment.akev")),
            minted::<crate::text::Quotation>(include_str!("text/quotation.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "git")]
    Namespace {
        name: "git",
        models: &[
            minted::<crate::git::CheckoutState>(include_str!("git/checkout_state.akev")),
            minted::<crate::git::FileAnchor>(include_str!("git/file_anchor.akev")),
            minted::<crate::git::Repository>(include_str!("git/repository.akev")),
            minted::<crate::git::WorkLocation>(include_str!("git/work_location.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "graph")]
    Namespace {
        name: "graph",
        models: &[
            minted::<crate::graph::DynEdge>(include_str!("graph/dyn_edge.akev")),
            minted::<crate::graph::EdgeKind>(include_str!("graph/edge_kind.akev")),
        ],
        edge_kinds: Some(MintedEdgeKinds {
            model: crate::graph::EdgeKind::descriptor,
            kinds: crate::graph::EDGE_KINDS,
            akev: include_str!("graph/edge_kinds.akev"),
        }),
    },
    #[cfg(feature = "cognition")]
    Namespace {
        name: "cognition",
        models: &[
            minted::<crate::cognition::Claim>(include_str!("cognition/claim.akev")),
            minted::<crate::cognition::Decision>(include_str!("cognition/decision.akev")),
            minted::<crate::cognition::Expectation>(include_str!("cognition/expectation.akev")),
            minted::<crate::cognition::Intention>(include_str!("cognition/intention.akev")),
            minted::<crate::cognition::Intuition>(include_str!("cognition/intuition.akev")),
            minted::<crate::cognition::Perspective>(include_str!("cognition/perspective.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "work")]
    Namespace {
        name: "work",
        models: &[
            minted::<crate::work::Scope>(include_str!("work/scope.akev")),
            minted::<crate::work::Submission>(include_str!("work/submission.akev")),
            minted::<crate::work::Task>(include_str!("work/task.akev")),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "messaging")]
    Namespace {
        name: "messaging",
        models: &[
            minted::<crate::messaging::Mention>(include_str!("messaging/mention.akev")),
            minted::<crate::messaging::Message>(include_str!("messaging/message.akev")),
        ],
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
        let names = |namespace: &Namespace| namespace.models.iter().map(|model| (model.descriptor)().name.to_owned()).collect();
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
