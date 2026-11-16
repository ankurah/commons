//! Mints the definitions of the Commons' models and the Commons' kinds of edge into
//! `.akev` files beside the models' structs, and checks those files. The crate's README
//! says what the check establishes and what the tool does not settle.

mod namespaces;
mod registering;
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
#[command(about = "Mints the definitions of the Commons' models and kinds of edge into .akev files, and checks them")]
enum Command {
    /// Register every model of every namespace on a throwaway node and create the kinds of
    /// edge, and write each model's definitions beside its struct and the edge kinds'
    /// records in the `graph` namespace, as `.akev` files.
    Mint {
        /// The namespaces' parent directory, which must hold no `.akev` file yet.
        #[arg(long, default_value_os_t = sources())]
        src: PathBuf,
    },
    /// Check the `.akev` files a mint keeps.
    Verify {
        /// The namespaces' parent directory.
        #[arg(long, default_value_os_t = sources())]
        src: PathBuf,
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
        Command::Mint { src } => mint(&src).await,
        Command::Verify { src } => verify(&src),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// The crate's own sources, where a kept mint lives.
fn sources() -> PathBuf {
    let mint = Path::new(env!("CARGO_MANIFEST_DIR"));
    mint.parent().expect("the mint crate sits in the workspace").join("src")
}

async fn mint(src: &Path) -> Result<(), CommandError> {
    writing::refuse_kept(src)?;
    let minted = registering::mint(NAMESPACES).await?;
    writing::write(src, &minted)?;

    let models: usize = minted.iter().map(|namespace| namespace.models.len()).sum();
    let properties: usize = minted.iter().flat_map(|namespace| &namespace.models).map(|model| model.properties.len()).sum();
    let edge_kinds: usize = minted.iter().map(|namespace| namespace.edge_kinds.len()).sum();
    println!(
        "Minted {models} models in {} namespaces, with {properties} properties and their links, and {edge_kinds} edge kinds, into {}",
        minted.len(),
        src.display()
    );
    Ok(())
}

fn verify(src: &Path) -> Result<(), CommandError> {
    let verified = verifying::verify(src, NAMESPACES)?;
    println!("Verified {} files, rebuilding {} creation events", verified.files, verified.events);
    Ok(())
}
