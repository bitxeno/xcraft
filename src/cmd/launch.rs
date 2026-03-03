use anyhow::Result;
use clap::Parser;

use crate::{build, launch};

use super::build::{BuildArgs, resolve_and_build};

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[derive(Parser)]
pub struct LaunchArgs {
    #[command(flatten)]
    build: BuildArgs,

    /// Bring simulator to foreground (default: true)
    #[arg(long, default_value_t = true)]
    foreground_simulator: bool,

    /// Launch arguments passed to the app (repeatable)
    #[arg(long = "arg")]
    launch_args: Vec<String>,

    /// Launch environment KEY=VALUE (repeatable)
    #[arg(long = "env", value_parser = parse_key_val)]
    launch_env: Vec<(String, String)>,

    /// Only install the app without launching it (simulator/device only)
    #[arg(long)]
    install_only: bool,
}

pub fn cmd_launch(args: LaunchArgs) -> Result<()> {
    let resolved = resolve_and_build(&args.build)?;

    // Get launch info from build settings.
    let info = build::get_launch_info(
        &resolved.effective_ws,
        &resolved.scheme_name,
        &resolved.config,
        &resolved.dest,
        args.build.action.derived_data.as_deref(),
    )?;

    eprintln!();
    eprintln!("App path:  {}", info.app_path.display());
    eprintln!("Bundle ID: {}", info.bundle_id);

    // Launch.
    let launch_opts = launch::LaunchOptions {
        dest: &resolved.dest,
        info: &info,
        args: &args.launch_args,
        env: &args.launch_env,
        foreground_simulator: args.foreground_simulator,
        install_only: args.install_only,
    };
    launch::launch(&launch_opts)?;

    Ok(())
}
