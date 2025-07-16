![logo](resources/icon_256x256.png)

# Wezlix

[WezTerm](https://wezfurlong.org/wezterm/) + [Helix](https://helix-editor.com/) = Wezlix, or WezTerm as a UI shell for Helix editor.

This repository contains:

- Git submodules for
  - the [wez/wezterm](https://github.com/wez/wezterm/)
  - a [fork](https://github.com/0x6b/helix) of Helix editor, based on [25.07](https://github.com/helix-editor/helix/releases/tag/25.07), with Japanese specific modifications ([diff](https://github.com/helix-editor/helix/compare/25.07...0x6b:helix:japanese-word-boundary))
- A simple launcher
- A build script to build both of them, and to create an application bundle for macOS

## Build

```
$ git clone --recursive https://github.com/0x6b/wezlix
$ cargo run -- --release
```

Please note that the
`--release` flag is an option for the build script,
`src/wezlix-builder.rs`, not for
`cargo run`.

## Usage

For macOS, the build process will create an application bundle
`Wezlix.app` in the
`target/app` directory, which you can use as a standalone application.

```console
$ ./target/app/Wezlix.app/Contents/MacOS/wezlix --help
Usage: wezlix [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Sets the input file to use

Options:
      --wezterm-config <WEZTERM_CONFIG>  Specifies a file to use for WezTerm configuration
      --helix-config <HELIX_CONFIG>      Specifies a file to use for Helix configuration
  -h, --help                             Print help

$ # or double-click ./target/app/Wezlix.app
```

Default configuration files are placed at:

-

## `$XDG_CONFIG_HOME/wezlix/wezlix.lua` for Wezlix-specific WezTerm configuration

## `$XDG_CONFIG_HOME/wezlix/helix.toml` for Helix configuration

`$XDG_CONFIG_HOME/wezlix/env.toml` for environment variables which will be set while launching the program i.e.

```toml
PATH = "/home/username/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
```

### Limitations

- The application bundle is not signed, so you will need to allow it to run in the security settings.
- Drag and drop won't do what you expect; dragging files from Finder.app to the Wezlix.app will pass the file names to the editor, but the editor will not open them.

## License

WezTerm and Helix are licensed under their own licenses. Other files are licensed under MIT. See [LICENSE](LICENSE).
