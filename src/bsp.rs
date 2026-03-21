use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::json;

use crate::cache::CachedState;

const BUILD_SERVER_JSON: &str = "buildServer.json";

/// Write a minimal `buildServer.json` with only BSP boilerplate and `argv`.
/// Used by `bsp config` to set up the initial file.
pub fn write_minimal_build_server_json(root: &Path, argv: Vec<String>) -> Result<()> {
    let config = json!({
        "name": "xcraft bsp",
        "version": env!("CARGO_PKG_VERSION"),
        "bspVersion": "2.0",
        "languages": ["c", "cpp", "objective-c", "objective-cpp", "swift"],
        "argv": argv,
    });

    let path = root.join(BUILD_SERVER_JSON);
    std::fs::write(&path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

/// Update `buildServer.json` with workspace/scheme/build_root from xcraft cached state.
/// Preserves existing fields (especially `argv`).
pub fn write_build_server_json(root: &Path, state: &CachedState) -> Result<()> {
    let path = root.join(BUILD_SERVER_JSON);
    let mut config: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path).context("failed to read buildServer.json")?,
    )
    .context("failed to parse buildServer.json")?;

    let map = config
        .as_object_mut()
        .context("buildServer.json is not a JSON object")?;

    // Inject workspace (absolute path).
    let ws_rel = state
        .bsp
        .as_ref()
        .and_then(|b| b.generated_workspace.as_deref())
        .or(state.workspace.as_deref());
    if let Some(ws) = ws_rel {
        let ws_abs = root.join(ws);
        map.insert(
            "workspace".into(),
            serde_json::Value::String(ws_abs.display().to_string()),
        );
    }

    // Inject scheme.
    if let Some(scheme) = &state.scheme {
        map.insert("scheme".into(), serde_json::Value::String(scheme.clone()));
    }

    // Inject build_root.
    if let Some(bsp) = &state.bsp
        && let Some(br) = &bsp.build_root
    {
        map.insert("build_root".into(), serde_json::Value::String(br.clone()));
    }

    map.insert("kind".into(), serde_json::Value::String("xcode".into()));

    std::fs::write(&path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

/// Forward BSP messages from `reader` to `writer`, flushing after each complete message.
///
/// BSP uses Content-Length framing (same as LSP):
/// ```text
/// Content-Length: <N>\r\n
/// \r\n
/// <N bytes of JSON-RPC>
/// ```
pub fn forward_messages(reader: impl std::io::Read, mut writer: impl Write) -> Result<()> {
    let mut reader = BufReader::new(reader);
    loop {
        // Read header line: "Content-Length: <N>\r\n"
        let mut header = String::new();
        let n = reader.read_line(&mut header)?;
        if n == 0 {
            break; // EOF
        }

        let length: usize = header
            .strip_prefix("Content-Length:")
            .and_then(|s| s.trim().parse().ok())
            .with_context(|| format!("invalid BSP header: {header:?}"))?;

        // Read blank line separator.
        let mut blank = String::new();
        reader.read_line(&mut blank)?;

        // Read body.
        let mut body = vec![0u8; length];
        reader.read_exact(&mut body)?;

        // Write header + blank line + body, then flush.
        write!(writer, "Content-Length: {length}\r\n\r\n")?;
        writer.write_all(&body)?;
        writer.flush()?;
    }
    Ok(())
}
