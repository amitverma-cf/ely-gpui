# Ely GPUI Component

A component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed.
Every component ships in light and dark. The palette is warm and quiet. Color carries meaning, not decoration.

Every component runs live at https://elygpui.com, compiled to WebAssembly.

Status: early. `TASKS.md` tracks every component, chapter by chapter. To help, see `CONTRIBUTING.md`.

## Use

```toml
[dependencies]
ely-gpui-component = { git = "https://git.zacharyzhang.com/ZacharyZhang-NY/Ely-GPUI-Components" }
gpui = { package = "gpui-pre", version = "=0.3.8" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.8", features = ["font-kit"] }
```

```rust
use ely_gpui_component::{Assets, theme::{Mode, Theme}};
use gpui::App;

fn main() {
    gpui_platform::application().with_assets(Assets).run(|cx: &mut App| {
        ely_gpui_component::init(cx).expect("Ely failed to start");
        Theme::set_mode(Mode::Dark, cx);
    });
}
```

`init` registers the fonts and the theme. It returns an error if the application's asset source cannot load one of Ely's files.

## Configuration (`ely-gpui.toml`)

Ely features a compile-time configuration manifest `ely-gpui.toml` located at your workspace root. It drives tokens, palettes, layout metrics, fonts, media, and icon pruning with **zero runtime deserialization overhead** (constants and static slices are baked at compilation into `$OUT_DIR`):

```toml
[paths]
assets_dir = "assets"
icons_dir = "assets/icons"
fonts_dir = "assets/fonts"
scan_sources = ["src/**/*.rs"]

[theme]
preset = "ely"

[theme.palette.light]
bg = 0xfcfaf7
fg = 0x181613
paper = 0xffffff
ink = 0x181613
# ... WCAG 2.1 AA (4.5:1) checked at build time

[theme.tokens]
base_rem = 16.0
sidebar_width_collapsed = 56.0
dialog_width = 440.0
titlebar_height = 38.0

[icons]
packs = ["lucide", "heroicons", "hugeicons"]
mode = "auto"     # evicts unused SVGs from compiled binary
minify = true     # minifies SVGs at compilation

[media.images]
logo = { path = "logo.png", compress = "optipng" }
```

## CLI (`cargo ely`)

Ely ships with a development CLI tool to scaffold projects, add components, and inspect configuration:

```sh
# Initialize ely-gpui.toml and asset directories (detects workspace vs single-crate layout)
cargo ely init

# Add an icon pack and scaffold provider directory
cargo ely add icons heroicons

# Inspect component catalog and usage
cargo ely add button

# Verify theme contrast compliance, font paths, and asset resolution
cargo ely doctor
```

## Typed Provider Icons & Dead Asset Eviction

Icons are referenced through provider-scoped enums rather than loose string paths:

```rust
use ely_gpui_component::primitives::{Icon, LucideIcon, HeroIcon, HugeIcon};

// Compile-time verified icon names
Icon::new(LucideIcon::ArrowRight)
Icon::new(HeroIcon::Sparkles)
Icon::new(HugeIcon::AiChat01)
```

At compilation, `build.rs` scans your source tree via AST matching (`scan_sources`) and only bakes the SVGs your views actually use into `EMBEDDED_ICONS`. Unreferenced icons add zero bytes to your compiled binary.

## Type-Safe Media & Fonts

Declared fonts and media in `ely-gpui.toml` generate compile-time verified enums:

```rust
use ely_gpui_component::assets::Image;

let path = Image::Logo.path();          // "logo.png"
let mime = Image::Logo.mime_type();     // "image/png"
let bytes = Image::Logo.bytes();        // &'static [u8] (compressed)
```

Images can be automatically compressed during compilation via `compress = "optipng"` or `"webp"`.

## Migration Guide

- **`IconName` Removal**: Replace `IconName::Name` with provider-scoped enums like `LucideIcon::Name`, `HeroIcon::Name`, or `HugeIcon::Name`.
- **String Icon Paths**: Use `Icon::new(LucideIcon::Variant)` instead of passing raw string paths to built-in components.
- **Parametric Themes**: Override layout dimensions and WCAG colors directly in `ely-gpui.toml` under `[theme.tokens]` and `[theme.palette]` rather than maintaining custom code wrappers.

## With GPUI Kit

Ely runs beside Longbridge's [GPUI Kit](https://gpui-kit.com) in one window, on the GPUI snapshot Kit 0.7.1 pins. Pass `Assets::before(gpui_kit::assets::Assets)` as the one asset source, call both `init`s, open the window with `gpui_kit::open_window`, and wrap the view in Ely's root `FocusScope`. The [guide](https://elygpui.com/docs/gpui-kit) covers themes, focus and the inspector; `compat/kit` holds a whole app and the tests behind each claim.

## Gallery

```sh
cargo run --example gallery
cargo run --example gallery -- --page buttons
cargo run --example gallery -- --capture shots   # macOS: PNG of every page, light and dark
ELY_GALLERY_ASSETS=https://example.com/gallery/assets/ scripts/web.sh dist   # for the browser
```

In a browser, `?page=buttons&story=icon-button&theme=dark` draws one section alone; `examples/gallery/stories.json` lists them all.

## Website

```sh
cd frontend
pnpm install
ELY_GALLERY_ASSETS=https://<site>/gallery/assets/ pnpm gallery   # the wasm gallery, into public/gallery
pnpm dev        # or: pnpm build && pnpm preview; both fetch the showcase at showcase.lock's commit
ELY_SHOWCASE=../../Ely-GPUI-Showcases pnpm dev   # the showcase from a local checkout
node scripts/og.mjs   # with pnpm preview up: public/og.png from the hero
node scripts/favicon.mjs   # public/favicon.svg, apple-touch-icon.png and ../logo.png from the mark
pnpm deploy     # Cloudflare, through wrangler
```

## Build notes

- gpui compiles its Metal shaders at runtime here (`runtime_shaders`, on by default), so a full Xcode install is not required. gpui_platform needs `font-kit`, or macOS draws no text.
- CI tests the crate and builds the gallery on macOS, Windows and Linux, arm64 and x86_64. The capture tool needs macOS.

## License

MIT. Lucide icons: ISC. Inter, JetBrains Mono, IBM Plex Sans and the gallery's Noto Sans Hebrew: SIL Open Font License 1.1.
The gallery photos in `examples/gallery/assets` were generated for this project with gpt-image-2.
