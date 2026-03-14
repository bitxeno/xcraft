use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use crate::{scheme, workspace};

#[derive(Serialize)]
struct SchemeEntry {
    name: String,
    arg: String,
}

pub fn cmd_schemes(ws_path: Option<PathBuf>, json: bool) -> Result<()> {
    let ws = workspace::resolve_workspace(ws_path.as_deref(), None)?;
    let effective_ws = ws.ensure_generated()?;
    let schemes = scheme::list_schemes(&effective_ws)?;
    if json {
        let entries: Vec<SchemeEntry> = schemes
            .iter()
            .map(|s| SchemeEntry {
                name: s.clone(),
                arg: s.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for s in &schemes {
            println!("{s}");
        }
    }
    Ok(())
}
