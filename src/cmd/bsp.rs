use std::io;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cmd::build::{ResolveArgs, resolve_and_cache};
use crate::{bsp, build, cache};

// ---------------------------------------------------------------------------
// bsp config
// ---------------------------------------------------------------------------

#[derive(Parser)]
pub struct BspConfigArgs {
    /// Ignore cached selections and re-prompt for all options
    #[arg(long)]
    pub configure: bool,

    #[command(flatten)]
    pub resolve: ResolveArgs,

    /// Path to derived data
    #[arg(long)]
    pub derived_data: Option<String>,
}

pub fn cmd_bsp_config(args: BspConfigArgs) -> Result<()> {
    let resolved = resolve_and_cache(&args.resolve, args.configure)?;
    let cache_root = cache::CachedState::root()?;
    let profile = args.resolve.profile.as_deref();

    // Get build settings to extract SYMROOT → compute build_root.
    let entries = build::get_build_settings(
        &resolved.effective_ws,
        &resolved.scheme_name,
        &resolved.config,
        None,
        args.derived_data.as_deref(),
    )?;
    let entry = entries
        .first()
        .context("no build settings returned by xcodebuild")?;
    let symroot = entry
        .build_settings
        .get("SYMROOT")
        .and_then(|v| v.as_str())
        .context("SYMROOT not found in build settings")?;

    // build_root = SYMROOT/../../ (strip Build/Products), matching xcode-build-server.
    let build_root = std::path::Path::new(symroot)
        .join("../..")
        .canonicalize()
        .context("failed to canonicalize build_root")?;

    // Compute generated_workspace (relative to cache root).
    let generated_workspace = resolved
        .effective_ws
        .path
        .strip_prefix(&cache_root)
        .unwrap_or(&resolved.effective_ws.path)
        .display()
        .to_string();

    // Save BSP state to cache.
    let mut state = cache::CachedState::load(&cache_root, profile);
    state.bsp = Some(cache::BspState {
        generated_workspace: Some(generated_workspace),
        build_root: Some(build_root.display().to_string()),
    });
    state.save(&cache_root, profile)?;

    // Build argv for buildServer.json.
    let exe = std::env::current_exe()?.canonicalize()?;
    let mut argv: Vec<String> = vec![exe.display().to_string(), "bsp".into(), "serve".into()];
    if let Some(p) = profile {
        argv.push("--profile".into());
        argv.push(p.to_string());
    }

    // Write minimal buildServer.json.
    bsp::write_minimal_build_server_json(&cache_root, argv)?;

    eprintln!("Build root:    {}", build_root.display());
    eprintln!("Wrote buildServer.json");
    eprintln!();

    if which_xcode_build_server().is_none() {
        eprintln!("Warning: xcode-build-server is not installed.");
        eprintln!("  Install it with: brew install xcode-build-server");
        eprintln!();
    }

    eprintln!(
        "Run `xcraft build` or `xcraft launch` to generate compile commands for SourceKit-LSP."
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// bsp serve
// ---------------------------------------------------------------------------

#[derive(Parser)]
pub struct BspServeArgs {
    /// Use a named profile for cached selections
    #[arg(long)]
    pub profile: Option<String>,
}

pub fn cmd_bsp_serve(args: BspServeArgs) -> Result<()> {
    let cache_root = cache::CachedState::root()?;
    let profile = args.profile.as_deref();
    let state = cache::CachedState::load(&cache_root, profile);

    if state.workspace.is_none() || state.scheme.is_none() {
        bail!("no cached state found; run `xcraft configure` or `xcraft bsp config` first");
    }

    // Update buildServer.json with current xcraft state.
    bsp::write_build_server_json(&cache_root, &state)?;

    // Spawn xcode-build-server and proxy stdin/stdout.
    let mut child = Command::new("xcode-build-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn xcode-build-server")?;

    let server_stdin = child.stdin.take().unwrap();
    let server_stdout = child.stdout.take().unwrap();

    // Server → Client: forward BSP messages, flushing after each.
    let fwd_thread = thread::spawn(move || -> Result<()> {
        let stdout = io::stdout().lock();
        bsp::forward_messages(server_stdout, stdout)
    });

    // Client → Server: forward BSP messages.
    let stdin = io::stdin().lock();
    let _ = bsp::forward_messages(stdin, server_stdin);

    let status = child.wait()?;
    let _ = fwd_thread.join();

    if !status.success() {
        bail!("xcode-build-server exited with {status}");
    }
    Ok(())
}

fn which_xcode_build_server() -> Option<()> {
    Command::new("which")
        .arg("xcode-build-server")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|_| ())
}
