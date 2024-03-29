use std::{
    collections::HashMap,
    env::{current_dir, current_exe},
    error::Error,
    fs,
    fs::read_to_string,
    path::PathBuf,
    process::Command,
};

use clap::Parser;
use home::home_dir;
use xdg::BaseDirectories;

#[derive(Parser)]
struct Args {
    /// Specifies a file to use for WezTerm configuration
    #[clap(long)]
    wezterm_config: Option<PathBuf>,

    /// Specifies a file to use for Helix configuration
    #[clap(long)]
    helix_config: Option<PathBuf>,

    /// Specifies a file to set environment variables
    #[clap(long)]
    env: Option<PathBuf>,

    /// Sets the input file to use
    files: Vec<PathBuf>,
}

type EnvironmentVariables = HashMap<String, String>;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let wezterm_config = match args.wezterm_config {
        Some(path) => path,
        None => BaseDirectories::with_prefix("wezlix")?.place_config_file("wezlix.lua")?,
    };
    let helix_config = match args.helix_config {
        Some(path) => path,
        None => BaseDirectories::with_prefix("wezlix")?.place_config_file("helix.toml")?,
    };
    let env = match args.env {
        Some(path) => path,
        None => BaseDirectories::with_prefix("wezlix")?.place_config_file("env.toml")?,
    };

    let bin_path = {
        let path = current_exe()?;

        // follow symlink
        let metadata = fs::symlink_metadata(&path)?;
        match metadata.file_type().is_symlink() {
            true => fs::read_link(path)?.canonicalize()?,
            false => path,
        }
    };
    let bin_root = bin_path.parent().unwrap();
    let current_dir = current_dir()?;
    let current_dir =
        if current_dir == PathBuf::from("/") { home_dir().unwrap() } else { current_dir };
    let env_vars: EnvironmentVariables = toml::from_str(&read_to_string(env)?)?;

    // run wezterm-gui in the parent directory of the bin_path
    let mut command = Command::new(bin_root.join("wezterm-gui"));
    command
        .arg("--config-file")
        .arg(wezterm_config)
        .arg("start")
        .arg("--cwd")
        .arg(current_dir)
        .arg(bin_root.join("hx"))
        .arg("--config")
        .arg(helix_config)
        .args(args.files)
        .envs(env_vars)
        .status()
        .unwrap();

    Ok(())
}
