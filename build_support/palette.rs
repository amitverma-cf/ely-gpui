use serde::Deserialize;

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

pub fn check_contrast(mode: &str, bg: u32, fg: u32) {
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

pub const PALETTE_FIELDS: &[&str] = &[
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

pub fn emit_palette_struct(code: &mut String) {
    code.push_str("#[derive(Clone, Copy, Debug)]\n");
    code.push_str("pub struct BakedPalette {\n");
    for f in PALETTE_FIELDS {
        code.push_str(&format!("    pub {}: u32,\n", f));
    }
    code.push_str("}\n\n");
}

pub fn emit_palette_instance(name: &str, p: &PaletteColors, code: &mut String) {
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

pub fn default_light_palette() -> PaletteColors {
    PaletteColors {
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
    }
}

pub fn default_dark_palette() -> PaletteColors {
    PaletteColors {
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
    }
}
