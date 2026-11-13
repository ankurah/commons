//! Mints the definitions of the Commons' models, and checks a minted directory. Whether a
//! mint is kept is the runner's choice; the crate's README says what `verify` establishes
//! and what the tool does not settle.

mod namespaces;
mod page;
mod registering;
mod state_encoding;
mod verifying;
mod writing;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use thiserror::Error;

use crate::{namespaces::NAMESPACES, registering::MintError, verifying::VerifyError, writing::WriteError};

#[derive(Parser)]
#[command(about = "Mints the definitions of the Commons' models, and checks a minted directory")]
enum Command {
    /// Register every model of every namespace on a throwaway node, and write each
    /// definition's creation event and each namespace's page.
    Mint {
        /// The directory to write into, which must not exist yet.
        #[arg(long, default_value_os_t = default_out())]
        out: PathBuf,
    },
    /// Check a directory `mint` wrote.
    Verify {
        /// The directory `mint` wrote.
        #[arg(long)]
        dir: PathBuf,
    },
}

/// Why a command failed.
#[derive(Debug, Error)]
enum CommandError {
    #[error("minting failed: {0}")]
    Mint(#[from] MintError),
    #[error("writing failed: {0}")]
    Write(#[from] WriteError),
    #[error("verification failed: {0}")]
    Verify(#[from] VerifyError),
}

#[tokio::main]
async fn main() -> ExitCode {
    let result = match Command::parse() {
        Command::Mint { out } => mint(&out).await,
        Command::Verify { dir } => verify(&dir),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// A trial run should leave nothing in the repository, so by default the output goes
/// under the workspace's `target/`, which git ignores; a mint that is to be kept is
/// written into the repository with `--out` and committed.
fn default_out() -> PathBuf {
    let mint = Path::new(env!("CARGO_MANIFEST_DIR"));
    mint.parent().expect("the mint crate sits in the workspace").join("target").join("mint")
}

async fn mint(out: &Path) -> Result<(), CommandError> {
    writing::refuse_existing(out)?;
    let minted = registering::mint(NAMESPACES).await?;
    writing::write(out, &minted)?;

    let models: usize = minted.iter().map(|namespace| namespace.models.len()).sum();
    let properties: usize = minted.iter().flat_map(|namespace| &namespace.models).map(|model| model.properties.len()).sum();
    println!("Minted {models} models in {} namespaces, with {properties} properties and their links, into {}", minted.len(), out.display());
    Ok(())
}

fn verify(dir: &Path) -> Result<(), CommandError> {
    let verified = verifying::verify(dir)?;
    println!("Verified {} definitions in {} directories", verified.definitions, verified.directories);
    Ok(())
}
