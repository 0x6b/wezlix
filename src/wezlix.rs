use std::{env::current_exe, error::Error, fs, process::Command};

use xdg::BaseDirectories;

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = BaseDirectories::with_prefix("wezlix")?.place_config_file("wezlix.lua")?;
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

    // run wezterm-gui in the parent directory of the bin_path
    Command::new(bin_root.join("wezterm-gui"))
        .arg("--config-file")
        .arg(config_path)
        .arg("start")
        .arg(bin_root.join("hx"))
        .spawn()?;

    Ok(())
}
