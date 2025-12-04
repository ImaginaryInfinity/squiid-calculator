use std::{fmt::Debug, path::PathBuf};

pub mod config;

pub mod backend;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Could not find section `{0}`")]
    MissingSection(String),
    #[error("Could not find key `{key}` in section `{section}`")]
    MissingKey { section: String, key: String },
    #[error("Attempted to mutate a malformed config")]
    MalformedConfig,
}

pub trait ConfigBackend: Debug + Send + Sync {
    fn load(&self) -> Option<String>;
    fn save(&self, content: &str) -> Result<(), String>;
    fn config_directory(&self) -> Option<PathBuf>;
}
