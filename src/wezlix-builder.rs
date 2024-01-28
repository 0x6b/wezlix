use std::{
    env::var,
    error::Error,
    fs::{create_dir_all, read, File},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use apple_bundle::{
    info_plist::InfoPlist,
    plist,
    prelude::{
        BundleVersion, Categorization, Graphics, Icons, Identification, Launch, Localization,
        MainUserInterface, Naming,
    },
};
use clap::Parser;
use fs_extra::{copy_items, dir::CopyOptions, remove_items};
use tiny_skia::{Pixmap, Transform};
use usvg::{Options, TreeParsing};

fn main() -> Result<(), Box<dyn Error>> {
    WezlixBuilder::new().build()
}

#[derive(Debug, Parser)]
pub struct WezlixBuilder {
    /// Build with release mode
    #[clap(long)]
    pub release: bool,

    /// Run `cargo clean` for the dependencies before building
    #[clap(long)]
    pub clean: bool,

    /// Path to cargo executable
    #[clap(long, env, default_value = "cargo")]
    pub cargo: String,

    #[clap(skip)]
    env: Environment,
}

#[derive(Debug, Default)]
pub struct Environment {
    pub app_name: &'static str,

    pub wezterm_root: PathBuf,
    pub helix_root: PathBuf,

    pub wezterm_release_dir: PathBuf,
    pub helix_release_dir: PathBuf,

    pub app_dir: PathBuf,
    pub app_contents_dir: PathBuf,
    pub app_binary_dir: PathBuf,
    pub app_extras_dir: PathBuf,
    pub app_icon: PathBuf,
}

impl WezlixBuilder {
    fn new() -> Self {
        let options = Self::parse();

        let app_name = "Wezlix.app";
        let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
        let target_dir = manifest_dir.join("target");
        let app_dir = target_dir.clone().join("app");
        let app_contents_dir = app_dir.join(app_name).join("Contents");
        let wezterm_root = manifest_dir.join("wezterm");
        let helix_root = manifest_dir.join("helix");
        let resource_dir = manifest_dir.join("resources");

        let env = Environment {
            app_name,

            wezterm_root,
            helix_root,

            wezterm_release_dir: target_dir.join(if options.release { "release" } else { "debug" }),
            helix_release_dir: target_dir.join(if options.release { "opt" } else { "debug" }),

            app_icon: resource_dir.join("wezlix.svg"),
            app_binary_dir: app_contents_dir.join("MacOS"),
            app_extras_dir: app_contents_dir.join("Resources"),
            app_contents_dir,
            app_dir,
        };

        WezlixBuilder { env, ..options }
    }

    fn build(&self) -> Result<(), Box<dyn Error>> {
        // Rust equivalent of https://github.com/alacritty/alacritty/blob/v0.13.1/Makefile
        if self.clean {
            println!("Cleaning app Wezterm and Helix dependencies");
            self.clean_dependency(&self.env.wezterm_root)?;
            self.clean_dependency(&self.env.helix_root)?;
        }

        println!("Building Wezterm");
        // wezterm-gui/build.rs expects the release directory to be in the same directory, but
        // we're building in a different directory. To work around the error while building in
        // release mode, we need to create the target directory.
        create_dir_all(self.env.wezterm_root.join("target").join("release"))?;
        self.build_dependency(
            &self.env.wezterm_root,
            if self.release { Some("release") } else { None },
        )?;
        println!("Building Helix");
        self.build_dependency(&self.env.helix_root, if self.release { Some("opt") } else { None })?;
        println!("Building Wezlix launcher");
        Command::new(&self.cargo)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .arg("build")
            .arg("--bin")
            .arg("wezlix")
            .arg("--release")
            .status()?;

        #[cfg(target_os = "macos")]
        {
            println!("Cleaning app package");
            self.clean_app_package()?;
            println!("Creating icons");
            self.create_icons()?;
            println!("Building app package");
            self.build_app_package()?;
            println!("Created '{}' in '{}'", self.env.app_name, self.env.app_dir.display());
        }

        Ok(())
    }

    fn clean_dependency<P>(&self, target: P) -> Result<ExitStatus, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        Ok(Command::new(&self.cargo)
            .current_dir(target)
            .env("CARGO_TARGET_DIR", "../target")
            .args(["clean"])
            .status()?)
    }

    fn build_dependency<P>(
        &self,
        target: P,
        profile: Option<&str>,
    ) -> Result<ExitStatus, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let args = match profile {
            Some(profile) => vec!["build", "--profile", profile],
            None => vec!["build"],
        };

        Ok(Command::new(&self.cargo)
            .current_dir(target)
            .env("CARGO_TARGET_DIR", "../target")
            .args(&args)
            .status()?)
    }

    fn clean_app_package(&self) -> Result<(), Box<dyn Error>> {
        remove_items(&[self.env.app_dir.join(self.env.app_name)])?;
        create_dir_all(&self.env.app_binary_dir)?;
        create_dir_all(&self.env.app_extras_dir)?;
        create_dir_all(self.env.app_binary_dir.join("runtime").join("queries"))?;
        create_dir_all(self.env.app_binary_dir.join("runtime").join("themes"))?;
        Ok(())
    }

    fn create_icons(&self) -> Result<(), Box<dyn Error>> {
        let icons = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("wezlix.iconset");
        create_dir_all(&icons)?;

        let rtree = resvg::Tree::from_usvg(&usvg::Tree::from_data(
            &read(&self.env.app_icon)?,
            &Options::default(),
        )?);

        [
            (16, "16x16"),
            (32, "16x16@2x"),
            (32, "32x32"),
            (64, "32x32@2x"),
            (128, "128x128"),
            (256, "128x128@2x"),
            (256, "256x256"),
            (512, "256x256@2x"),
            (512, "512x512"),
            (1024, "512x512@2x"),
        ]
        .iter()
        .for_each(|(s, name)| {
            let mut pixmap = Pixmap::new(*s, *s).unwrap();
            rtree.render(
                Transform::from_scale(
                    *s as f32 / rtree.size.width(),
                    *s as f32 / rtree.size.height(),
                ),
                &mut pixmap.as_mut(),
            );
            pixmap.save_png(icons.join(format!("icon_{}.png", name))).unwrap();
        });

        #[cfg(target_os = "macos")]
        Command::new("iconutil")
            .args([
                "--convert",
                "icns",
                icons.to_string_lossy().to_string().as_str(),
                "--output",
                self.env.app_extras_dir.join("wezlix.icns").to_string_lossy().as_ref(),
            ])
            .spawn()
            .expect("Failed to create icns file");

        Ok(())
    }

    fn build_app_package(&self) -> Result<(), Box<dyn Error>> {
        let copy_options = CopyOptions {
            overwrite: true,
            skip_exist: false,
            buffer_size: 64000,
            copy_inside: true,
            depth: 0,
            content_only: false,
        };

        let properties = InfoPlist {
            localization: Localization {
                bundle_development_region: Some("en".to_owned()),
                ..Default::default()
            },
            launch: Launch {
                bundle_executable: Some("wezlix".to_owned()),
                ..Default::default()
            },
            identification: Identification {
                bundle_identifier: "io.warpnine.wezlix".to_owned(),
                ..Default::default()
            },
            bundle_version: BundleVersion {
                bundle_version: Some("1".to_owned()),
                bundle_info_dictionary_version: Some("1.0".to_owned()),
                bundle_short_version_string: Some(env!("CARGO_PKG_VERSION").to_owned()),
                ..Default::default()
            },
            naming: Naming {
                bundle_name: Some("Wezlix".to_owned()),
                bundle_display_name: Some("Wezlix".to_owned()),
                ..Default::default()
            },
            icons: Icons {
                bundle_icon_file: Some("wezlix.icns".to_owned()),
                ..Default::default()
            },
            graphics: Graphics {
                high_resolution_capable: Some(true),
                supports_automatic_graphics_switching: Some(true),
                ..Default::default()
            },
            main_user_interface: MainUserInterface {
                main_nib_file_base_name: None,
                ..Default::default()
            },

            categorization: Categorization {
                bundle_package_type: Some("APPL".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        };
        // Create Info.plist file
        plist::to_writer_xml(
            File::create(self.env.app_contents_dir.join("Info.plist"))?,
            &properties,
        )
        .unwrap();

        copy_items(
            &[
                self.env.helix_release_dir.join("hx"),
                self.env.wezterm_release_dir.join("wezlix"),
                self.env.wezterm_release_dir.join("wezterm"),
                self.env.wezterm_release_dir.join("wezterm-mux-server"),
                self.env.wezterm_release_dir.join("wezterm-gui"),
                self.env.wezterm_release_dir.join("strip-ansi-escapes"),
            ],
            &self.env.app_binary_dir,
            &copy_options,
        )?;

        copy_items(
            &[
                self.env
                    .wezterm_root
                    .join("assets")
                    .join("macos")
                    .join("WezTerm.app")
                    .join("libEGL.dylib"),
                self.env
                    .wezterm_root
                    .join("assets")
                    .join("macos")
                    .join("WezTerm.app")
                    .join("libGLESv1_CM.dylib"),
                self.env
                    .wezterm_root
                    .join("assets")
                    .join("macos")
                    .join("WezTerm.app")
                    .join("libGLESv2.dylib"),
            ],
            self.env.app_dir.join(self.env.app_name),
            &copy_options,
        )?;

        copy_items(
            &[
                self.env.helix_root.join("runtime").join("queries"),
                self.env.helix_root.join("runtime").join("themes"),
                self.env.helix_root.join("runtime").join("tutor"),
            ],
            self.env.app_binary_dir.join("runtime"),
            &copy_options,
        )?;

        Ok(())
    }
}
