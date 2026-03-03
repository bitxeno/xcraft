use std::path::PathBuf;

use anyhow::Result;

use crate::{scheme, workspace};

pub fn cmd_schemes(ws_path: Option<PathBuf>) -> Result<()> {
    let ws = workspace::resolve_workspace(ws_path.as_deref(), None)?;
    let effective_ws = ws.ensure_generated()?;
    let schemes = scheme::list_schemes(&effective_ws)?;
    for s in &schemes {
        println!("{s}");
    }
    Ok(())
}
