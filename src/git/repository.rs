use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// A git repository.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.repository")]
pub struct Repository {
    #[active_type(LWW)]
    pub name: String,
    /// An array of URLs.
    pub remote_urls: Json,
    #[active_type(LWW)]
    pub default_branch: String,
}
