use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Subcommand)]
pub enum Command {
    /// Get the string value of a given string key
    Get { key: String },
    /// Set the value of a string key to a string
    Set { key: String, value: String },
    /// Remove a given key
    Rm { key: String },
}

impl Command {
    pub fn set(key: String, value: String) -> Self {
        Self::Set { key, value }
    }
    pub fn rm(key: String) -> Self {
        Self::Rm { key }
    }
}

#[derive(Clone, Debug)]
pub struct CommandPos {
    pub(crate) offset: u64,
    pub(crate) size: u64,
}
