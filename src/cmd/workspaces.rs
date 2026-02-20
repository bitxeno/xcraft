use anyhow::Result;

use crate::workspace;

pub fn cmd_workspaces() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let workspaces = workspace::detect_workspaces(&cwd);
    if workspaces.is_empty() {
        eprintln!("No workspaces found.");
    }
    for ws in &workspaces {
        let tag = match ws.ws_type {
            workspace::WorkspaceType::Xcode => "xcode",
            workspace::WorkspaceType::Spm => "spm",
        };
        println!("[{tag}] {}", ws.path.display());
    }
    Ok(())
}
