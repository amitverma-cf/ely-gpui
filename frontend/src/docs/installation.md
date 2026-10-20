# Installation

## Requirements

- Rust 1.95 or later, the version Ely's `Cargo.toml` names.
- macOS, Windows, or Linux (all tested in CI).
- No full Xcode: GPUI compiles its Metal shaders at run time.

## Add the dependencies

Ely is not on crates.io. Depend on it by git, with GPUI at the snapshot Ely pins. `gpui-pre` republishes Zed's GPUI on crates.io, and GPUI Kit pins the same one:

```toml
[dependencies]
ely-gpui = { git = "https://github.com/amitverma-cf/ely-gpui" }
gpui = { package = "gpui-pre", version = "=0.3.9" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.9", features = ["font-kit"] }
```

| Line | Why |
| --- | --- |
| `ely-gpui` | The library. `Cargo.lock` records the commit; add `rev = "<commit>"` to hold it in `Cargo.toml` as well. |
| `gpui` | The same snapshot as Ely's, under the name `gpui`. Another builds a second GPUI, and its types do not match the ones Ely takes. |
| `gpui_platform` | Opens windows on each platform. Without `font-kit`, macOS falls back to GPUI's no-op text system and draws no text. |

## Shaders

Ely's default feature, `runtime_shaders`, turns on the same feature in `gpui_platform`, so GPUI compiles its Metal shaders at run time. With `default-features = false` the build runs `xcrun metal` instead, which needs Xcode's Metal compiler.

## The `cargo ely` tool

`cargo ely` sets a project up and copies components into it as source. From a clone of the repository:

```sh
cargo install --path cli
cd your-project
cargo ely init
cargo ely add button
```

`init` writes `ely-gpui.toml` and `.cargo/config.toml`, and downloads Inter, JetBrains Mono, IBM Plex Sans and the Lucide icons into `assets/`, each checked against a pinned checksum. See [the CLI guide](/docs/#the-cli) for every command.

## Project configuration

`ely-gpui.toml` at the project root defines design tokens, paths, media and icon pruning. The build reads it through `ELY_CONFIG_PATH`, which `init` sets in `.cargo/config.toml`:

```toml
[paths]
scan_sources = ["src/**/*.rs"]

[icons]
mode = "auto"
```

## Check the install

Copy the [first window](/docs/#a-first-window) into `src/main.rs` and run it:

```sh
cargo run
```

A window opens with a counter and a Press button. From a clone of the repository, `cargo run --example gallery` opens every chapter.

## When it fails

| Symptom | Cause and fix |
| --- | --- |
| Build error: `ely: ELY_CONFIG_PATH is not set` | The project has no `ely-gpui.toml` for the build to read. Run `cargo ely init`. |
| `init` error: `ely: the asset source lacks N of Ely's files, first <path>: <reason>` | The app's source cannot load every file Ely bundles. Pass `Assets`, or `Assets::before(yours)` when the app has a source of its own. |
| `init` error: `ely: embedded fonts failed to register` | GPUI's text system refused Ely's fonts. File a bug with the log. |
| Windows open with no text on macOS | `gpui_platform` lacks the `font-kit` feature. |
| `mismatched types`, with the note ``there are multiple different versions of crate `gpui` in the dependency graph`` | `gpui` names another snapshot or a git source. Use `gpui-pre` at `=0.3.9`. |
| `metal shader compilation failed` | Default features are off and Xcode's Metal compiler is missing. Keep `runtime_shaders`. |
