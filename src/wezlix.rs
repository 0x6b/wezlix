use std::{
    env::{current_dir, current_exe},
    error::Error,
    fs,
    path::PathBuf,
    process::Command,
};

use clap::Parser;
use xdg::BaseDirectories;

#[derive(Parser)]
struct Args {
    /// Specifies a file to use for WezTerm configuration
    #[clap(long)]
    wezterm_config: Option<PathBuf>,

    /// Specifies a file to use for Helix configuration
    #[clap(long)]
    helix_config: Option<PathBuf>,

    /// Sets the input file to use
    files: Vec<PathBuf>,
}

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
        if current_dir == PathBuf::from("/") { home::home_dir().unwrap() } else { current_dir };

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
        .status()
        .unwrap();

    Ok(())
}
