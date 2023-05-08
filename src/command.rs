use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    Get { key: String },
    Set { key: String, value: String },
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
