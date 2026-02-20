use std::path::PathBuf;

use anyhow::Result;

use crate::{scheme, workspace};

pub fn cmd_configs(ws_path: Option<PathBuf>) -> Result<()> {
    let ws = workspace::resolve_workspace(ws_path.as_deref())?;
    let configs = scheme::list_configurations(&ws)?;
    for c in &configs {
        println!("{c}");
    }
    Ok(())
}
