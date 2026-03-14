use anyhow::Result;
use serde::Serialize;

use crate::workspace;

#[derive(Serialize)]
struct WorkspaceEntry {
    #[serde(rename = "type")]
    ws_type: String,
    path: String,
    arg: String,
}

pub fn cmd_workspaces(json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let workspaces = workspace::detect_workspaces(&cwd);
    if workspaces.is_empty() {
        eprintln!("No workspaces found.");
    }
    if json {
        let entries: Vec<WorkspaceEntry> = workspaces
            .iter()
            .map(|ws| {
                let ws_type = match ws.ws_type {
                    workspace::WorkspaceType::Xcode => "xcode",
                    workspace::WorkspaceType::Spm => "spm",
                    workspace::WorkspaceType::Tuist => "tuist",
                };
                WorkspaceEntry {
                    ws_type: ws_type.to_string(),
                    path: ws.path.display().to_string(),
                    arg: ws.path.display().to_string(),
                }
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for ws in &workspaces {
            println!("{ws}");
        }
    }
    Ok(())
}
