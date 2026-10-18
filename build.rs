#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct Config {
    paths: Option<PathsConfig>,
    theme: Option<ThemeConfig>,
    fonts: Option<FontsConfig>,
    icons: Option<IconsConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct PathsConfig {
    assets_dir: Option<String>,
    icons_dir: Option<String>,
    fonts_dir: Option<String>,
    images_dir: Option<String>,
    scan_sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct ThemeConfig {
    default_mode: Option<String>,
    high_contrast: Option<bool>,
    color_blind_safe: Option<bool>,
    density: Option<String>,
    radius_scale: Option<f32>,
    font_scale: Option<f32>,
    palette: Option<String>,
    #[serde(default)]
    palette_light: Option<PaletteColors>,
    #[serde(default)]
    palette_dark: Option<PaletteColors>,
    tokens: Option<TokensConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct PaletteColors {
    bg: Option<u32>,
    surface: Option<u32>,
    sunken: Option<u32>,
    overlay: Option<u32>,
    hover: Option<u32>,
    active: Option<u32>,
    border: Option<u32>,
    border_strong: Option<u32>,
    fg: Option<u32>,
    fg_muted: Option<u32>,
    fg_subtle: Option<u32>,
    fg_disabled: Option<u32>,
    accent: Option<u32>,
    accent_hover: Option<u32>,
    on_accent: Option<u32>,
    on_media: Option<u32>,
    focus: Option<u32>,
    link: Option<u32>,
    selection: Option<u32>,
    success: Option<u32>,
    warning: Option<u32>,
    danger: Option<u32>,
    info: Option<u32>,
    success_subtle: Option<u32>,
    warning_subtle: Option<u32>,
    danger_subtle: Option<u32>,
    info_subtle: Option<u32>,
    backdrop: Option<u32>,
    media_backdrop: Option<u32>,
    shimmer: Option<u32>,
    glass: Option<u32>,
    shadow: Option<u32>,
    tooltip_bg: Option<u32>,
    tooltip_fg: Option<u32>,
    paper: Option<u32>,
    ink: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
struct TokensConfig {
    base_rem: Option<f32>,
    radius_sm: Option<f32>,
    radius_md: Option<f32>,
    radius_lg: Option<f32>,
    radius_xl: Option<f32>,
    text_xs: Option<f32>,
    text_sm: Option<f32>,
    text_base: Option<f32>,
    text_md: Option<f32>,
    text_lg: Option<f32>,
    text_xl: Option<f32>,
    text_xxl: Option<f32>,
    text_display: Option<f32>,
    control_height_sm: Option<f32>,
    control_height_md: Option<f32>,
    control_height_lg: Option<f32>,
    control_padding_sm: Option<f32>,
    control_padding_md: Option<f32>,
    control_padding_lg: Option<f32>,
    table_row_compact: Option<f32>,
    table_row_standard: Option<f32>,
    table_row_comfortable: Option<f32>,
    sidebar_width_collapsed: Option<f32>,
    sidebar_width_expanded: Option<f32>,
    sheet_size: Option<f32>,
    dialog_width: Option<f32>,
    toast_width: Option<f32>,
    titlebar_height: Option<f32>,
    pane_min: Option<f32>,
    hairline: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
struct FontsConfig {
    sans: Option<FontEntry>,
    mono: Option<FontEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct FontEntry {
    family: Option<String>,
    files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct IconsConfig {
    packs: Option<Vec<String>>,
    mode: Option<String>,
    minify: Option<bool>,
    fallback: Option<String>,
}

fn find_config_file() -> Option<PathBuf> {
    if let Ok(path) = env::var("ELY_CONFIG_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut curr = manifest_dir.as_path();
    loop {
        let candidate = curr.join("ely-gpui.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        match curr.parent() {
            Some(parent) => curr = parent,
            None => break,
        }
    }
    None
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(rgb_hex: u32) -> f32 {
    let r = ((rgb_hex >> 16) & 0xff) as f32 / 255.0;
    let g = ((rgb_hex >> 8) & 0xff) as f32 / 255.0;
    let b = (rgb_hex & 0xff) as f32 / 255.0;
    0.2126 * srgb_to_linear(r) + 0.7152 * srgb_to_linear(g) + 0.0722 * srgb_to_linear(b)
}

fn check_contrast(mode: &str, bg: u32, fg: u32) {
    let l_bg = relative_luminance(bg);
    let l_fg = relative_luminance(fg);
    let (lighter, darker) = if l_bg > l_fg {
        (l_bg, l_fg)
    } else {
        (l_fg, l_bg)
    };
    let ratio = (lighter + 0.05) / (darker + 0.05);
    if ratio < 4.5 {
        println!(
            "cargo:warning=ely: WCAG AA contrast ratio is {:.2}:1 (< 4.5:1) for {} mode (bg: 0x{:06x}, fg: 0x{:06x})",
            ratio, mode, bg, fg
        );
    }
}

fn emit_palette_struct(code: &mut String) {
    code.push_str("#[derive(Clone, Copy, Debug)]\n");
    code.push_str("pub struct BakedPalette {\n");
    let fields = [
        "bg",
        "surface",
        "sunken",
        "overlay",
        "hover",
        "active",
        "border",
        "border_strong",
        "fg",
        "fg_muted",
        "fg_subtle",
        "fg_disabled",
        "accent",
        "accent_hover",
        "on_accent",
        "on_media",
        "focus",
        "link",
        "selection",
        "success",
        "warning",
        "danger",
        "info",
        "success_subtle",
        "warning_subtle",
        "danger_subtle",
        "info_subtle",
        "backdrop",
        "media_backdrop",
        "shimmer",
        "glass",
        "shadow",
        "tooltip_bg",
        "tooltip_fg",
        "paper",
        "ink",
    ];
    for f in fields {
        code.push_str(&format!("    pub {f}: u32,\n"));
    }
    code.push_str("}\n\n");
}

fn emit_palette_instance(name: &str, p: &PaletteColors, code: &mut String) {
    code.push_str(&format!(
        "pub const {}: BakedPalette = BakedPalette {{\n",
        name
    ));
    macro_rules! f {
        ($field:ident, $default:expr) => {
            code.push_str(&format!(
                "    {}: 0x{:08x},\n",
                stringify!($field),
                p.$field.unwrap_or($default)
            ));
        };
    }
    f!(bg, 0xfcfaf7);
    f!(surface, 0xffffff);
    f!(sunken, 0xf6f4f1);
    f!(overlay, 0xffffff);
    f!(hover, 0xf2f0ed);
    f!(active, 0xe9e7e5);
    f!(border, 0xe1dfdd);
    f!(border_strong, 0xcccac8);
    f!(fg, 0x181613);
    f!(fg_muted, 0x605d5a);
    f!(fg_subtle, 0x696764);
    f!(fg_disabled, 0xacaaa8);
    f!(accent, 0x181613);
    f!(accent_hover, 0x302d2b);
    f!(on_accent, 0xfcfaf7);
    f!(on_media, 0xffffff);
    f!(focus, 0x3772bb);
    f!(link, 0x2863ab);
    f!(selection, 0x3772bb33);
    f!(success, 0x267b4c);
    f!(warning, 0xa25f12);
    f!(danger, 0xba3e38);
    f!(info, 0x2e69b2);
    f!(success_subtle, 0xe7f9ec);
    f!(warning_subtle, 0xfef2dd);
    f!(danger_subtle, 0xffefec);
    f!(info_subtle, 0xebf4ff);
    f!(backdrop, 0x18161352);
    f!(media_backdrop, 0x0e0d0bf8);
    f!(shimmer, 0xffffffb3);
    f!(glass, 0xfcfaf7b8);
    f!(shadow, 0x181613);
    f!(tooltip_bg, 0x181613);
    f!(tooltip_fg, 0xfcfaf7);
    f!(paper, 0xffffff);
    f!(ink, 0x181613);
    code.push_str("};\n\n");
}

fn main() {
    println!("cargo:rerun-if-env-changed=ELY_CONFIG_PATH");

    let config_path = find_config_file();
    let config = if let Some(ref path) = config_path {
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(path).unwrap_or_default();
        toml::from_str::<Config>(&content).unwrap_or_default()
    } else {
        Config::default()
    };

    let theme = config.theme.as_ref();
    let default_mode = theme
        .and_then(|t| t.default_mode.as_deref())
        .unwrap_or("system");
    let high_contrast = theme.and_then(|t| t.high_contrast).unwrap_or(false);
    let color_blind_safe = theme.and_then(|t| t.color_blind_safe).unwrap_or(false);
    let density_str = theme
        .and_then(|t| t.density.as_deref())
        .unwrap_or("standard");
    let radius_scale = theme.and_then(|t| t.radius_scale).unwrap_or(1.0);
    let font_scale = theme.and_then(|t| t.font_scale).unwrap_or(1.0);

    let light_palette = theme
        .and_then(|t| t.palette_light.clone())
        .unwrap_or_default();
    let dark_palette = theme
        .and_then(|t| t.palette_dark.clone())
        .unwrap_or(PaletteColors {
            bg: Some(0x252525),
            surface: Some(0x2c2c2c),
            sunken: Some(0x202020),
            overlay: Some(0x323232),
            hover: Some(0x383838),
            active: Some(0x3f3f3f),
            border: Some(0x3f3f3f),
            border_strong: Some(0x555555),
            fg: Some(0xf3f1f0),
            fg_muted: Some(0xc4c2c0),
            fg_subtle: Some(0xacaaa8),
            fg_disabled: Some(0x5c5a58),
            accent: Some(0xf3f1f0),
            accent_hover: Some(0xd9d7d5),
            on_accent: Some(0x252525),
            on_media: Some(0xffffff),
            focus: Some(0x69a1e8),
            link: Some(0x7eb1f3),
            selection: Some(0x69a1e84d),
            success: Some(0x6bbc89),
            warning: Some(0xe8b45e),
            danger: Some(0xe5756e),
            info: Some(0x77abec),
            success_subtle: Some(0x182c1f),
            warning_subtle: Some(0x342611),
            danger_subtle: Some(0x3a1d1b),
            info_subtle: Some(0x19273a),
            backdrop: Some(0x00000080),
            media_backdrop: Some(0x000000f8),
            shimmer: Some(0xffffff10),
            glass: Some(0x252525b8),
            shadow: Some(0x000000),
            tooltip_bg: Some(0x4a4a4a),
            tooltip_fg: Some(0xf3f1f0),
            paper: Some(0xf3f1f0),
            ink: Some(0x181613),
        });

    let light_bg = light_palette.bg.unwrap_or(0xfcfaf7);
    let light_fg = light_palette.fg.unwrap_or(0x181613);
    let dark_bg = dark_palette.bg.unwrap_or(0x252525);
    let dark_fg = dark_palette.fg.unwrap_or(0xf3f1f0);

    check_contrast("light", light_bg, light_fg);
    check_contrast("dark", dark_bg, dark_fg);

    let tokens = theme.and_then(|t| t.tokens.as_ref());
    let base_rem = tokens.and_then(|t| t.base_rem).unwrap_or(16.0);
    let r_sm = tokens.and_then(|t| t.radius_sm).unwrap_or(4.0);
    let r_md = tokens.and_then(|t| t.radius_md).unwrap_or(6.0);
    let r_lg = tokens.and_then(|t| t.radius_lg).unwrap_or(8.0);
    let r_xl = tokens.and_then(|t| t.radius_xl).unwrap_or(12.0);

    let text_xs = tokens.and_then(|t| t.text_xs).unwrap_or(11.0);
    let text_sm = tokens.and_then(|t| t.text_sm).unwrap_or(12.0);
    let text_base = tokens.and_then(|t| t.text_base).unwrap_or(13.0);
    let text_md = tokens.and_then(|t| t.text_md).unwrap_or(14.0);
    let text_lg = tokens.and_then(|t| t.text_lg).unwrap_or(16.0);
    let text_xl = tokens.and_then(|t| t.text_xl).unwrap_or(20.0);
    let text_xxl = tokens.and_then(|t| t.text_xxl).unwrap_or(24.0);
    let text_display = tokens.and_then(|t| t.text_display).unwrap_or(32.0);

    let ch_sm = tokens.and_then(|t| t.control_height_sm).unwrap_or(24.0);
    let ch_md = tokens.and_then(|t| t.control_height_md).unwrap_or(28.0);
    let ch_lg = tokens.and_then(|t| t.control_height_lg).unwrap_or(32.0);
    let cp_sm = tokens.and_then(|t| t.control_padding_sm).unwrap_or(8.0);
    let cp_md = tokens.and_then(|t| t.control_padding_md).unwrap_or(12.0);
    let cp_lg = tokens.and_then(|t| t.control_padding_lg).unwrap_or(16.0);

    let tr_compact = tokens.and_then(|t| t.table_row_compact).unwrap_or(28.0);
    let tr_std = tokens.and_then(|t| t.table_row_standard).unwrap_or(36.0);
    let tr_comf = tokens.and_then(|t| t.table_row_comfortable).unwrap_or(44.0);

    let fonts = config.fonts.as_ref();
    let sans_fam = fonts
        .and_then(|f| f.sans.as_ref()?.family.as_deref())
        .unwrap_or("Inter");
    let mono_fam = fonts
        .and_then(|f| f.mono.as_ref()?.family.as_deref())
        .unwrap_or("JetBrains Mono");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ely_baked.rs");

    let mut code = String::new();
    code.push_str("// Auto-generated by Ely GPUI build.rs. Do not edit.\n\n");
    code.push_str(&format!(
        "pub const BAKED_DEFAULT_MODE: &str = {:?};\n",
        default_mode
    ));
    code.push_str(&format!(
        "pub const BAKED_HIGH_CONTRAST: bool = {};\n",
        high_contrast
    ));
    code.push_str(&format!(
        "pub const BAKED_COLOR_BLIND_SAFE: bool = {};\n",
        color_blind_safe
    ));
    code.push_str(&format!(
        "pub const BAKED_DENSITY_STR: &str = {:?};\n",
        density_str
    ));
    code.push_str(&format!(
        "pub const BAKED_RADIUS_SCALE: f32 = {:.4};\n",
        radius_scale
    ));
    code.push_str(&format!(
        "pub const BAKED_FONT_SCALE: f32 = {:.4};\n",
        font_scale
    ));
    code.push_str(&format!(
        "pub const BAKED_FONT_SANS: &str = {:?};\n",
        sans_fam
    ));
    code.push_str(&format!(
        "pub const BAKED_FONT_MONO: &str = {:?};\n",
        mono_fam
    ));

    code.push_str(&format!(
        "pub const BAKED_BASE_REM: f32 = {:.4};\n",
        base_rem
    ));
    code.push_str(&format!("pub const BAKED_RADIUS_SM: f32 = {:.4};\n", r_sm));
    code.push_str(&format!("pub const BAKED_RADIUS_MD: f32 = {:.4};\n", r_md));
    code.push_str(&format!("pub const BAKED_RADIUS_LG: f32 = {:.4};\n", r_lg));
    code.push_str(&format!("pub const BAKED_RADIUS_XL: f32 = {:.4};\n", r_xl));

    code.push_str(&format!("pub const BAKED_TEXT_XS: f32 = {:.4};\n", text_xs));
    code.push_str(&format!("pub const BAKED_TEXT_SM: f32 = {:.4};\n", text_sm));
    code.push_str(&format!(
        "pub const BAKED_TEXT_BASE: f32 = {:.4};\n",
        text_base
    ));
    code.push_str(&format!("pub const BAKED_TEXT_MD: f32 = {:.4};\n", text_md));
    code.push_str(&format!("pub const BAKED_TEXT_LG: f32 = {:.4};\n", text_lg));
    code.push_str(&format!("pub const BAKED_TEXT_XL: f32 = {:.4};\n", text_xl));
    code.push_str(&format!(
        "pub const BAKED_TEXT_XXL: f32 = {:.4};\n",
        text_xxl
    ));
    code.push_str(&format!(
        "pub const BAKED_TEXT_DISPLAY: f32 = {:.4};\n",
        text_display
    ));

    code.push_str(&format!(
        "pub const BAKED_CONTROL_HEIGHT_SM: f32 = {:.4};\n",
        ch_sm
    ));
    code.push_str(&format!(
        "pub const BAKED_CONTROL_HEIGHT_MD: f32 = {:.4};\n",
        ch_md
    ));
    code.push_str(&format!(
        "pub const BAKED_CONTROL_HEIGHT_LG: f32 = {:.4};\n",
        ch_lg
    ));
    code.push_str(&format!(
        "pub const BAKED_CONTROL_PADDING_SM: f32 = {:.4};\n",
        cp_sm
    ));
    code.push_str(&format!(
        "pub const BAKED_CONTROL_PADDING_MD: f32 = {:.4};\n",
        cp_md
    ));
    code.push_str(&format!(
        "pub const BAKED_CONTROL_PADDING_LG: f32 = {:.4};\n",
        cp_lg
    ));
    code.push_str(&format!(
        "pub const BAKED_TABLE_ROW_COMPACT: f32 = {:.4};\n",
        tr_compact
    ));
    code.push_str(&format!(
        "pub const BAKED_TABLE_ROW_STANDARD: f32 = {:.4};\n",
        tr_std
    ));
    code.push_str(&format!(
        "pub const BAKED_TABLE_ROW_COMFORTABLE: f32 = {:.4};\n\n",
        tr_comf
    ));

    emit_palette_struct(&mut code);
    emit_palette_instance("BAKED_LIGHT_PALETTE", &light_palette, &mut code);
    emit_palette_instance("BAKED_DARK_PALETTE", &dark_palette, &mut code);

    fs::write(&dest_path, code).expect("failed to write ely_baked.rs");
}
