use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ThemeConfig {
    pub default_mode: Option<String>,
    pub high_contrast: Option<bool>,
    pub color_blind_safe: Option<bool>,
    pub density: Option<String>,
    pub radius_scale: Option<f32>,
    pub font_scale: Option<f32>,
    pub preset: Option<String>,
    #[serde(default)]
    pub palette: Option<ThemePaletteConfig>,
    pub tokens: Option<TokensConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ThemePaletteConfig {
    pub light: Option<PaletteColors>,
    pub dark: Option<PaletteColors>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PaletteColors {
    pub bg: Option<u32>,
    pub surface: Option<u32>,
    pub sunken: Option<u32>,
    pub overlay: Option<u32>,
    pub hover: Option<u32>,
    pub active: Option<u32>,
    pub border: Option<u32>,
    pub border_strong: Option<u32>,
    pub fg: Option<u32>,
    pub fg_muted: Option<u32>,
    pub fg_subtle: Option<u32>,
    pub fg_disabled: Option<u32>,
    pub accent: Option<u32>,
    pub accent_hover: Option<u32>,
    pub on_accent: Option<u32>,
    pub on_media: Option<u32>,
    pub focus: Option<u32>,
    pub link: Option<u32>,
    pub selection: Option<u32>,
    pub success: Option<u32>,
    pub warning: Option<u32>,
    pub danger: Option<u32>,
    pub info: Option<u32>,
    pub success_subtle: Option<u32>,
    pub warning_subtle: Option<u32>,
    pub danger_subtle: Option<u32>,
    pub info_subtle: Option<u32>,
    pub backdrop: Option<u32>,
    pub media_backdrop: Option<u32>,
    pub shimmer: Option<u32>,
    pub glass: Option<u32>,
    pub shadow: Option<u32>,
    pub tooltip_bg: Option<u32>,
    pub tooltip_fg: Option<u32>,
    pub paper: Option<u32>,
    pub ink: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct TokensConfig {
    pub base_rem: Option<f32>,
    pub radius_sm: Option<f32>,
    pub radius_md: Option<f32>,
    pub radius_lg: Option<f32>,
    pub radius_xl: Option<f32>,
    pub text_xs: Option<f32>,
    pub text_sm: Option<f32>,
    pub text_base: Option<f32>,
    pub text_md: Option<f32>,
    pub text_lg: Option<f32>,
    pub text_xl: Option<f32>,
    pub text_xxl: Option<f32>,
    pub text_display: Option<f32>,
    pub control_height_sm: Option<f32>,
    pub control_height_md: Option<f32>,
    pub control_height_lg: Option<f32>,
    pub control_padding_sm: Option<f32>,
    pub control_padding_md: Option<f32>,
    pub control_padding_lg: Option<f32>,
    pub table_row_compact: Option<f32>,
    pub table_row_standard: Option<f32>,
    pub table_row_comfortable: Option<f32>,
    pub sidebar_width_collapsed: Option<f32>,
    pub sidebar_width_expanded: Option<f32>,
    pub sheet_size: Option<f32>,
    pub dialog_width: Option<f32>,
    pub toast_width: Option<f32>,
    pub titlebar_height: Option<f32>,
    pub pane_min: Option<f32>,
    pub hairline: Option<f32>,
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
        code.push_str(&format!("    pub {}: u32,\n", f));
    }
    code.push_str("}\n\n");
}

fn emit_palette_instance(name: &str, p: &PaletteColors, code: &mut String) {
    code.push_str(&format!(
        "pub const {}: BakedPalette = BakedPalette {{\n",
        name
    ));
    let fields: [(&str, Option<u32>); 36] = [
        ("bg", p.bg),
        ("surface", p.surface),
        ("sunken", p.sunken),
        ("overlay", p.overlay),
        ("hover", p.hover),
        ("active", p.active),
        ("border", p.border),
        ("border_strong", p.border_strong),
        ("fg", p.fg),
        ("fg_muted", p.fg_muted),
        ("fg_subtle", p.fg_subtle),
        ("fg_disabled", p.fg_disabled),
        ("accent", p.accent),
        ("accent_hover", p.accent_hover),
        ("on_accent", p.on_accent),
        ("on_media", p.on_media),
        ("focus", p.focus),
        ("link", p.link),
        ("selection", p.selection),
        ("success", p.success),
        ("warning", p.warning),
        ("danger", p.danger),
        ("info", p.info),
        ("success_subtle", p.success_subtle),
        ("warning_subtle", p.warning_subtle),
        ("danger_subtle", p.danger_subtle),
        ("info_subtle", p.info_subtle),
        ("backdrop", p.backdrop),
        ("media_backdrop", p.media_backdrop),
        ("shimmer", p.shimmer),
        ("glass", p.glass),
        ("shadow", p.shadow),
        ("tooltip_bg", p.tooltip_bg),
        ("tooltip_fg", p.tooltip_fg),
        ("paper", p.paper),
        ("ink", p.ink),
    ];
    for (f, val) in fields {
        let v = val.unwrap_or(0);
        code.push_str(&format!("    {}: 0x{:08x},\n", f, v));
    }
    code.push_str("};\n\n");
}

pub fn generate_tokens(
    config: Option<&ThemeConfig>,
    fonts: Option<&crate::build_support::fonts::FontsConfig>,
    out_dir: &Path,
) {
    let dest_path = out_dir.join("ely_baked.rs");
    let mut code = String::new();
    code.push_str("// Generated by build_support/tokens.rs. Do not edit.\n\n");

    let default_theme = ThemeConfig::default();
    let theme = config.unwrap_or(&default_theme);

    let default_mode = theme.default_mode.as_deref().unwrap_or("system");
    let high_contrast = theme.high_contrast.unwrap_or(false);
    let color_blind_safe = theme.color_blind_safe.unwrap_or(false);
    let density = theme.density.as_deref().unwrap_or("comfortable");
    let radius_scale = theme.radius_scale.unwrap_or(1.0);
    let font_scale = theme.font_scale.unwrap_or(1.0);
    let palette_name = theme.preset.as_deref().unwrap_or("ely");

    let font_sans = fonts
        .and_then(|f| f.sans.as_ref())
        .and_then(|e| e.family.as_deref())
        .unwrap_or("Inter");
    let font_mono = fonts
        .and_then(|f| f.mono.as_ref())
        .and_then(|e| e.family.as_deref())
        .unwrap_or("JetBrains Mono");

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
    code.push_str("#[allow(dead_code)]\n");
    code.push_str(&format!("pub const BAKED_DENSITY: &str = {:?};\n", density));
    code.push_str(&format!(
        "pub const BAKED_DENSITY_STR: &str = {:?};\n",
        density
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
        font_sans
    ));
    code.push_str(&format!(
        "pub const BAKED_FONT_MONO: &str = {:?};\n",
        font_mono
    ));
    code.push_str("#[allow(dead_code)]\n");
    code.push_str(&format!(
        "pub const BAKED_PALETTE_NAME: &str = {:?};\n",
        palette_name
    ));

    let default_light = PaletteColors {
        bg: Some(0xfbf9f5),
        surface: Some(0xf4f0ea),
        sunken: Some(0xede7de),
        overlay: Some(0xffffff),
        hover: Some(0xe5ded3),
        active: Some(0xdcd3c5),
        border: Some(0xdcd3c5),
        border_strong: Some(0xc2b7a3),
        fg: Some(0x181613),
        fg_muted: Some(0x56524a),
        fg_subtle: Some(0x827c70),
        fg_disabled: Some(0xb3aba0),
        accent: Some(0x181613),
        accent_hover: Some(0x2d2b28),
        on_accent: Some(0xfbf9f5),
        on_media: Some(0xffffff),
        focus: Some(0x2863ab),
        link: Some(0x205493),
        selection: Some(0x2863ab33),
        success: Some(0x226c43),
        warning: Some(0x9a5b13),
        danger: Some(0xb33934),
        info: Some(0x235e9b),
        success_subtle: Some(0xe5f2ea),
        warning_subtle: Some(0xfcf0df),
        danger_subtle: Some(0xfae8e7),
        info_subtle: Some(0xe5f0fa),
        backdrop: Some(0x18161366),
        media_backdrop: Some(0x181613eb),
        shimmer: Some(0xffffff66),
        glass: Some(0xfbf9f5cc),
        shadow: Some(0x181613),
        tooltip_bg: Some(0x181613),
        tooltip_fg: Some(0xfcfaf7),
        paper: Some(0xffffff),
        ink: Some(0x181613),
    };

    let default_dark = PaletteColors {
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
        fg_disabled: Some(0x686664),
        accent: Some(0xf3f1f0),
        accent_hover: Some(0xdedcdb),
        on_accent: Some(0x181613),
        on_media: Some(0xffffff),
        focus: Some(0x5c95dc),
        link: Some(0x6ca3e6),
        selection: Some(0x5c95dc40),
        success: Some(0x489d6e),
        warning: Some(0xc48134),
        danger: Some(0xdb605a),
        info: Some(0x508bcd),
        success_subtle: Some(0x1c3024),
        warning_subtle: Some(0x332617),
        danger_subtle: Some(0x341e1d),
        info_subtle: Some(0x1b2837),
        backdrop: Some(0x0e0d0b80),
        media_backdrop: Some(0x0e0d0bf8),
        shimmer: Some(0xffffff1a),
        glass: Some(0x252525b8),
        shadow: Some(0x0e0d0b),
        tooltip_bg: Some(0xf3f1f0),
        tooltip_fg: Some(0x181613),
        paper: Some(0x2c2c2c),
        ink: Some(0xf3f1f0),
    };

    let mut light_palette = default_light;
    if let Some(overrides) = theme.palette.as_ref().and_then(|p| p.light.as_ref()) {
        macro_rules! merge_light {
            ($($field:ident),*) => {
                $(if overrides.$field.is_some() { light_palette.$field = overrides.$field; })*
            };
        }
        merge_light!(
            bg,
            surface,
            sunken,
            overlay,
            hover,
            active,
            border,
            border_strong,
            fg,
            fg_muted,
            fg_subtle,
            fg_disabled,
            accent,
            accent_hover,
            on_accent,
            on_media,
            focus,
            link,
            selection,
            success,
            warning,
            danger,
            info,
            success_subtle,
            warning_subtle,
            danger_subtle,
            info_subtle,
            backdrop,
            media_backdrop,
            shimmer,
            glass,
            shadow,
            tooltip_bg,
            tooltip_fg,
            paper,
            ink
        );
    }

    let mut dark_palette = default_dark;
    if let Some(overrides) = theme.palette.as_ref().and_then(|p| p.dark.as_ref()) {
        macro_rules! merge_dark {
            ($($field:ident),*) => {
                $(if overrides.$field.is_some() { dark_palette.$field = overrides.$field; })*
            };
        }
        merge_dark!(
            bg,
            surface,
            sunken,
            overlay,
            hover,
            active,
            border,
            border_strong,
            fg,
            fg_muted,
            fg_subtle,
            fg_disabled,
            accent,
            accent_hover,
            on_accent,
            on_media,
            focus,
            link,
            selection,
            success,
            warning,
            danger,
            info,
            success_subtle,
            warning_subtle,
            danger_subtle,
            info_subtle,
            backdrop,
            media_backdrop,
            shimmer,
            glass,
            shadow,
            tooltip_bg,
            tooltip_fg,
            paper,
            ink
        );
    }

    if let (Some(bg), Some(fg)) = (light_palette.bg, light_palette.fg) {
        check_contrast("light", bg, fg);
    }
    if let (Some(bg), Some(fg)) = (dark_palette.bg, dark_palette.fg) {
        check_contrast("dark", bg, fg);
    }

    let default_tok = TokensConfig::default();
    let tok = theme.tokens.as_ref().unwrap_or(&default_tok);

    let base_rem = tok.base_rem.unwrap_or(16.0);
    let r_sm = tok.radius_sm.unwrap_or(4.0) * radius_scale;
    let r_md = tok.radius_md.unwrap_or(6.0) * radius_scale;
    let r_lg = tok.radius_lg.unwrap_or(8.0) * radius_scale;
    let r_xl = tok.radius_xl.unwrap_or(12.0) * radius_scale;

    let text_xs = tok.text_xs.unwrap_or(11.0) * font_scale;
    let text_sm = tok.text_sm.unwrap_or(12.0) * font_scale;
    let text_base = tok.text_base.unwrap_or(13.0) * font_scale;
    let text_md = tok.text_md.unwrap_or(14.0) * font_scale;
    let text_lg = tok.text_lg.unwrap_or(16.0) * font_scale;
    let text_xl = tok.text_xl.unwrap_or(20.0) * font_scale;
    let text_xxl = tok.text_xxl.unwrap_or(24.0) * font_scale;
    let text_display = tok.text_display.unwrap_or(32.0) * font_scale;

    let ch_sm = tok.control_height_sm.unwrap_or(24.0);
    let ch_md = tok.control_height_md.unwrap_or(28.0);
    let ch_lg = tok.control_height_lg.unwrap_or(32.0);

    let cp_sm = tok.control_padding_sm.unwrap_or(8.0);
    let cp_md = tok.control_padding_md.unwrap_or(12.0);
    let cp_lg = tok.control_padding_lg.unwrap_or(16.0);

    let tr_compact = tok.table_row_compact.unwrap_or(28.0);
    let tr_std = tok.table_row_standard.unwrap_or(36.0);
    let tr_comf = tok.table_row_comfortable.unwrap_or(44.0);

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

    fs::write(dest_path, code).expect("failed to write ely_baked.rs");
}
