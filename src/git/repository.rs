use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// A git repository.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.repository")]
#[model(id = "fxNrHi50qgqHknTgutk7hcW4XEH1B-czuTZ1TLoTaNg")]
#[model(no_ffi)]
pub struct Repository {
    #[active_type(LWW)]
    #[property(id = "kqNic0jOHCe3TTK9fQ8XJ9dqqGyRI-Bw3xiA-CyBrqU")]
    pub name: String,
    /// An array of URLs.
    #[property(id = "D5rR5YhBAW8Ih0Ky3yPZM_-DCeGdGi61ThNWh5F7nDE")]
    pub remote_urls: Json,
    #[active_type(LWW)]
    #[property(id = "OCYYRQ-gzPS2FsSmzOuxd5BuGl-dxYDYBBa-9c-3A2Q")]
    pub default_branch: String,
}
