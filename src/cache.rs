use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::destination::Destination;

const CACHE_DIR: &str = ".sweetpad";
const CACHE_FILE: &str = "state.toml";

/// Persisted state from the last `launch` invocation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CachedState {
    pub workspace: Option<String>,
    pub scheme: Option<String>,
    pub configuration: Option<String>,
    pub destination: Option<Destination>,
}

impl CachedState {
    /// Load cached state from `.sweetpad/state.toml` relative to `root`.
    pub fn load(root: &Path) -> Self {
        let path = root.join(CACHE_DIR).join(CACHE_FILE);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save cached state to `.sweetpad/state.toml` relative to `root`.
    pub fn save(&self, root: &Path) -> Result<()> {
        let dir = root.join(CACHE_DIR);
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(dir.join(CACHE_FILE), content)?;
        Ok(())
    }

    /// Root directory for the cache (current working directory).
    pub fn root() -> Result<PathBuf> {
        Ok(std::env::current_dir()?)
    }
}
