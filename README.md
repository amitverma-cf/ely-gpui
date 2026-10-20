# Ely GPUI (`ely-gpui`)

A component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed.
Every component ships in light and dark. The palette is warm and quiet. Color carries meaning, not decoration.

Every component runs live at https://elygpui.com, compiled to WebAssembly.

Status: early. `TASKS.md` tracks every component, chapter by chapter. To help, see `CONTRIBUTING.md`.

## Use

Ely works like shadcn/ui: the `ely-gpui` crate holds the foundation (theme, motion, primitives, i18n, layout, typography), and `cargo ely` copies each component into your project as source you own. Fonts and icons are not bundled: `init` downloads them into your `assets/`, and the build embeds only what you use.

```sh
cargo install --path cli      # from a clone of this repository: installs `cargo ely`
cd your-project
cargo ely init                # ely-gpui.toml, .cargo/config.toml, fonts and Lucide into assets/
cargo ely add button dialog   # source into src/ui, with the crates and files it needs
```

Then add `mod ui;` to your crate and start it:

```rust
mod ui;

use ely_gpui::{Assets, theme::{Mode, Theme}};
use gpui::App;

fn main() {
    gpui_platform::application().with_assets(Assets).run(|cx: &mut App| {
        ely_gpui::init(cx).expect("Ely failed to start");
        ui::init(cx);
        Theme::set_mode(Mode::Dark, cx);
    });
}
```

`cargo ely add` also runs `cargo add` for `ely-gpui`, `gpui` and the other crates the copied files use. `init` registers the fonts and the theme, and returns an error if the application's asset source cannot load one of Ely's files.

## Configuration (`ely-gpui.toml`)

`ely-gpui.toml` sits at your project root, and `.cargo/config.toml` points the build at it (`ELY_CONFIG_PATH`; `cargo ely init` writes both). It drives tokens, palettes, layout metrics, fonts, media, data files and icon pruning with **no runtime deserialization** (constants and static slices are baked at compile time into `$OUT_DIR`). A build without it fails and says so.

```toml
[paths]
assets_dir = "assets"
icons_dir = "assets/icons"
scan_sources = ["src/**/*.rs"]

[theme]
preset = "ely"

[theme.palette.light]
bg = 0xfcfaf7
fg = 0x181613
# ... WCAG 2.1 AA (4.5:1) checked at build time

[theme.tokens]
base_rem = 16.0
dialog_width = 440.0

[icons]
mode = "auto"     # embeds only the icons your code names
minify = true

[media.images]
logo = { path = "logo.png", compress = "optipng" }

[media.files]     # data files a component loads; `cargo ely add` fills it in
countries = { path = "maps/countries.geojson" }
```

## The CLI (`cargo ely`)

```sh
cargo ely init                  # config, fonts and Lucide into assets/ (checksummed)
cargo ely list                  # every component, with what it needs
cargo ely list search chart     # find one
cargo ely add button            # copy it and its dependencies into src/ui
cargo ely add icons heroicons   # another icon pack into assets/icons
cargo ely doctor                # contrast, fonts, packs
cargo ely registry --check      # in this repo: is registry.json current?
```

`add` follows the real `use` graph of the source, rewrites imports (`crate::theme` becomes `ely_gpui::theme`, `crate::forms` becomes `crate::ui::forms`), and records each file's hash in `ely.lock`, so running it again keeps your edits (`--force` replaces them). Some entries are not copyable: they live in the `ely-gpui` crate, belong to another component, or are native to gpui; `add` says which.

## Built-in Theme Presets

Ely ships with carefully tuned, WCAG AA compliant light and dark presets:
- `ely` (warm paper & ink, default)
- `zinc` (neutral modern technical gray)
- `slate` (cool bluish slate)
- `nord` (arctic oceanic frost)
- `catppuccin` (Latte & Macchiato pastel hues)
- `tokyo-night` (Day & Storm neon indigo)

Choose a preset in `ely-gpui.toml`:
```toml
[theme]
preset = "nord"
```

Or switch dynamically at runtime:
```rust
Theme::set_preset("tokyo-night", cx);
```

## High-Level Layout & Command Overlays

- **`AppLayout`**: Desktop shell layout organizing titlebar header, left primary sidebar, right inspector panel, scrollable body, and status bar.
- **`CommandPalette`**: Fast modal quick-search and command execution overlay (<kbd>⌘K</kbd> / <kbd>Ctrl+K</kbd>) with keyboard navigation, grouped items, and shortcut badges.

## Typed Provider Icons & Dead Asset Eviction

Icons are referenced through provider-scoped enums rather than loose string paths:

```rust
use ely_gpui::primitives::{Icon, LucideIcon};

Icon::new(LucideIcon::ArrowRight)
```

`HeroIcon` and `HugeIcon` exist once `cargo ely add icons heroicons` (or `hugeicons`) has fetched the pack; before that they have no variants. At compile time `build.rs` scans your sources and the library's own core modules, and bakes only the SVGs they name into `EMBEDDED_ICONS`. The rest of each pack adds nothing to the binary.
