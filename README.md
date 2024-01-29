![logo](resources/icon_256x256.png)

# Wezlix

[WezTerm](https://wezfurlong.org/wezterm/) + [Helix](https://helix-editor.com/) = Wezlix, or WezTerm as a UI shell for Helix editor.

This repository contains:

- Git submodules for
  - the [wez/wezterm](https://github.com/wez/wezterm/)
  - a [fork](https://github.com/0x6b/helix) of Helix editor, based on [23.10](https://github.com/helix-editor/helix/releases/tag/23.10), with Japanese specific modifications ([diff](https://github.com/helix-editor/helix/compare/23.10...0x6b:helix:japanese-word-boundary))
- A simple launcher
- A build script to build both of them, and to create an application bundle for macOS

## Build

```
$ git clone --recursive https://github.com/0x6b/wezlix
$ cargo run -- --release
```

Please note that the `--release` flag is an option for the build script, `src/main.rs`, not for `cargo run`.

## Usage

```console
$ ./target/app/Wezlix.app/Contents/MacOS/wezlix [FILES]
$ # or double-click ./target/app/Wezlix.app
```

For macOS, the build process will create an application bundle `Wezlix.app` in the `target/app` directory, which you can use as a standalone application.

### Limitations

- The application bundle is not signed, so you will need to allow it to run in the security settings.
- Drag and drop won't do what you expect; dragging files from Finder.app to the Wezlix.app will pass the file names to the editor, but the editor will not open them.

## License

WezTerm and Helix are licensed under their own licenses. Other files are licensed under MIT. See [LICENSE](LICENSE).
