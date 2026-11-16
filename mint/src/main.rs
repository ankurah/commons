//! Mints the definitions of the Commons' models and the Commons' kinds of edge into
//! `.akev` files beside the models' structs, and checks those files. The crate's README
//! says what the check establishes and what the tool does not settle.

mod minting;
mod verifying;
mod writing;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use ankurah_commons::namespaces::NAMESPACES;
use clap::Parser;
use thiserror::Error;

use crate::{minting::MintError, verifying::VerifyError, writing::WriteError};

#[derive(Parser)]
#[command(about = "Mints the definitions of the Commons' models and kinds of edge into .akev files, and checks them")]
enum Command {
    /// Mint the definitions of every model of every namespace, and the records of the kinds
    /// of edge, into `.akev` files beside the structs that declare them. An entity that the
    /// kept mint there holds keeps its time and nonce.
    Mint {
        /// The namespaces' parent directory.
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

fn main() -> ExitCode {
    let result = match Command::parse() {
        Command::Mint { src } => mint(&src),
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

/// The crate's own sources, where the kept mint lives.
fn sources() -> PathBuf {
    let mint = Path::new(env!("CARGO_MANIFEST_DIR"));
    mint.parent().expect("the mint crate sits in the workspace").join("src")
}

fn mint(src: &Path) -> Result<(), CommandError> {
    let minted = minting::mint(src, NAMESPACES)?;
    writing::write(src, &minted)?;

    let models: usize = minted.iter().map(|namespace| namespace.models.len()).sum();
    let properties: usize = minted.iter().flat_map(|namespace| &namespace.models).map(|model| model.properties.len()).sum();
    let edge_kinds: usize = minted.iter().flat_map(|namespace| &namespace.edge_kinds).map(|edge_kinds| edge_kinds.kinds.len()).sum();
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
