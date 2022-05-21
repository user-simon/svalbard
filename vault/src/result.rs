//! Defines the [Error](crate::result::Error) and [Result] types that may be returned by the
//! [Vault](crate::Vault) API.

use std::{path::PathBuf, io, result};

use thiserror::Error;

/// Contains all errors and their respective messages.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Vault name '{0}' already exists. Try a different name")]
    VaultNameConflict(String),

    #[error("Seed index {0} out-of-bounds. This is a bug, please report to Mr. Simon.")]
    SeedIndex(usize),
    
    #[error("{1}: {0}")]
    IO(io::Error, PathBuf),

    #[error("Could not parse JSON in {1}. Attempt to fix manually and retry: {0}")]
    JSON(serde_json::Error, PathBuf),
}

/// Result type using the Svalbard [Error](crate::result::Error) enum.
pub type Result<T> = result::Result<T, Error>;
