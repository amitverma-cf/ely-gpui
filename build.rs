use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

mod build_support;

use build_support::fonts::FontsConfig;
use build_support::media::MediaConfig;
use build_support::tokens::ThemeConfig;

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct Config {
    paths: Option<PathsConfig>,
    theme: Option<ThemeConfig>,
    fonts: Option<FontsConfig>,
    icons: Option<IconsConfig>,
    media: Option<MediaConfig>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct PathsConfig {
    assets_dir: Option<String>,
    icons_dir: Option<String>,
    fonts_dir: Option<String>,
    images_dir: Option<String>,
    scan_sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
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

fn main() {
    println!("cargo:rerun-if-changed=ely-gpui.toml");
    println!("cargo:rerun-if-env-changed=ELY_CONFIG_PATH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let config_path = find_config_file();
    let config: Config = if let Some(path) = &config_path {
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(path).expect("failed to read ely-gpui.toml");
        toml::from_str(&content).expect("failed to parse ely-gpui.toml")
    } else {
        Config::default()
    };

    let root_dir = config_path
        .as_deref()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);

    let assets_dir = config
        .paths
        .as_ref()
        .and_then(|p| p.assets_dir.as_deref())
        .map(|d| root_dir.join(d))
        .unwrap_or_else(|| root_dir.join("assets"));

    // 1. Bake theme tokens and palettes
    build_support::tokens::generate_tokens(config.theme.as_ref(), config.fonts.as_ref(), &out_dir);

    // 2. Bake and verify fonts
    build_support::fonts::generate_fonts(config.fonts.as_ref(), root_dir, &assets_dir, &out_dir);

    // 3. Bake and verify media
    build_support::media::generate_media(config.media.as_ref(), root_dir, &assets_dir, &out_dir);

    // 4. Discover and prune icons
    let icons_dir = config
        .paths
        .as_ref()
        .and_then(|p| p.icons_dir.as_deref())
        .map(|d| root_dir.join(d))
        .unwrap_or_else(|| root_dir.join("assets/icons"));

    let scan_sources = config
        .paths
        .as_ref()
        .and_then(|p| p.scan_sources.clone())
        .unwrap_or_else(|| vec!["src".into(), "examples".into()]);

    let icon_mode = config
        .icons
        .as_ref()
        .and_then(|i| i.mode.as_deref())
        .unwrap_or("auto");

    let minify_icons = config.icons.as_ref().and_then(|i| i.minify).unwrap_or(true);

    build_support::icons::generate_icons(
        &icons_dir,
        &scan_sources,
        root_dir,
        &out_dir,
        icon_mode,
        minify_icons,
    );
}
