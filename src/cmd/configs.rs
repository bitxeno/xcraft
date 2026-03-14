use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use crate::{scheme, workspace};

#[derive(Serialize)]
struct ConfigEntry {
    name: String,
    arg: String,
}

pub fn cmd_configs(ws_path: Option<PathBuf>, json: bool) -> Result<()> {
    let ws = workspace::resolve_workspace(ws_path.as_deref(), None)?;
    let effective_ws = ws.ensure_generated()?;
    let configs = scheme::list_configurations(&effective_ws)?;
    if json {
        let entries: Vec<ConfigEntry> = configs
            .iter()
            .map(|c| ConfigEntry {
                name: c.clone(),
                arg: c.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for c in &configs {
            println!("{c}");
        }
    }
    Ok(())
}
