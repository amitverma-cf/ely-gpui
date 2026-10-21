use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct MediaConfig {
    #[serde(default)]
    pub images: BTreeMap<String, MediaImageEntry>,
    #[serde(default)]
    pub videos: BTreeMap<String, MediaVideoEntry>,
    /// Data files served by their path, with no enum: `maps/countries.geojson`.
    #[serde(default)]
    pub files: BTreeMap<String, MediaFileEntry>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MediaFileEntry {
    pub path: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MediaImageEntry {
    pub path: String,
    pub compress: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MediaVideoEntry {
    pub path: String,
    pub embed: Option<bool>,
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn mime_from_extension(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        _ => "application/octet-stream",
    }
}

fn compress_image(
    key: &str,
    src: &Path,
    mode: &str,
    out_dir: &Path,
    default_mime: &str,
) -> (PathBuf, String) {
    let media_out = out_dir.join("media");
    let _ = fs::create_dir_all(&media_out);
    match mode.to_ascii_lowercase().as_str() {
        "png" | "optipng" => {
            let dest = media_out.join(format!("{}.png", key));
            if let Ok(img) = image::open(src)
                && let Ok(file) = fs::File::create(&dest)
            {
                let writer = std::io::BufWriter::new(file);
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    writer,
                    image::codecs::png::CompressionType::Best,
                    image::codecs::png::FilterType::Adaptive,
                );
                if img.write_with_encoder(encoder).is_ok() {
                    if mode.eq_ignore_ascii_case("optipng") {
                        let _ = std::process::Command::new("optipng")
                            .arg("-quiet")
                            .arg("-o5")
                            .arg(&dest)
                            .status();
                    }
                    if let (Ok(dest_meta), Ok(src_meta)) = (fs::metadata(&dest), fs::metadata(src))
                        && dest_meta.len() >= src_meta.len()
                    {
                        let _ = fs::copy(src, &dest);
                    }
                    return (dest, "image/png".to_string());
                }
            }
            (src.to_path_buf(), default_mime.to_string())
        }
        "jpeg" | "jpg" => {
            let dest = media_out.join(format!("{}.jpg", key));
            if let Ok(img) = image::open(src)
                && let Ok(file) = fs::File::create(&dest)
            {
                let writer = std::io::BufWriter::new(file);
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(writer, 85);
                if img.write_with_encoder(encoder).is_ok() {
                    return (dest, "image/jpeg".to_string());
                }
            }
            (src.to_path_buf(), default_mime.to_string())
        }
        "webp" => {
            let dest = media_out.join(format!("{}.webp", key));
            let cwebp_ok = std::process::Command::new("cwebp")
                .arg("-q")
                .arg("85")
                .arg(src)
                .arg("-o")
                .arg(&dest)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if cwebp_ok && dest.exists() {
                return (dest, "image/webp".to_string());
            }
            let (fallback_dest, _) = compress_image(key, src, "png", out_dir, default_mime);
            (fallback_dest, "image/png".to_string())
        }
        _ => (src.to_path_buf(), default_mime.to_string()),
    }
}

pub fn generate_media(
    config: Option<&MediaConfig>,
    root_dir: &Path,
    assets_dir: &Path,
    out_dir: &Path,
) {
    let default_config = MediaConfig::default();
    let media = config.unwrap_or(&default_config);

    let mut resolved_images: Vec<(String, String, PathBuf, String)> = Vec::new();
    for (key, entry) in &media.images {
        let variant = to_pascal_case(key);
        let in_assets = assets_dir.join(&entry.path);
        let in_root = root_dir.join(&entry.path);
        let resolved = if in_assets.exists() {
            in_assets
        } else if in_root.exists() {
            in_root
        } else {
            panic!(
                "mok: Image file '{}' for key '{}' declared in mok-gpui.toml was not found on disk.\n\
                 Searched locations:\n\
                   - {}\n\
                   - {}\n\
                 Please verify the image exists or update your mok-gpui.toml [media.images] configuration.",
                entry.path,
                key,
                in_assets.display(),
                in_root.display()
            );
        };
        let ext = resolved
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let mime = mime_from_extension(&ext).to_string();
        let (final_path, final_mime) = if let Some(ref comp) = entry.compress {
            compress_image(key, &resolved, comp, out_dir, &mime)
        } else {
            (resolved, mime)
        };
        resolved_images.push((variant, entry.path.clone(), final_path, final_mime));
    }

    let mut resolved_videos: Vec<(String, String, PathBuf, bool)> = Vec::new();
    for (key, entry) in &media.videos {
        let variant = to_pascal_case(key);
        let should_embed = entry.embed.unwrap_or(true);
        let in_assets = assets_dir.join(&entry.path);
        let in_root = root_dir.join(&entry.path);
        let resolved = if in_assets.exists() {
            in_assets
        } else if in_root.exists() {
            in_root
        } else if should_embed {
            panic!(
                "mok: Video file '{}' for key '{}' declared in mok-gpui.toml was not found on disk.\n\
                 Searched locations:\n\
                   - {}\n\
                   - {}\n\
                 Please verify the video exists or set embed = false.",
                entry.path,
                key,
                in_assets.display(),
                in_root.display()
            );
        } else {
            in_root
        };
        resolved_videos.push((variant, entry.path.clone(), resolved, should_embed));
    }

    let mut resolved_files: Vec<(String, PathBuf)> = Vec::new();
    for (key, entry) in &media.files {
        let in_assets = assets_dir.join(&entry.path);
        let in_root = root_dir.join(&entry.path);
        let resolved = if in_assets.exists() {
            in_assets
        } else if in_root.exists() {
            in_root
        } else {
            panic!(
                "mok: file '{}' for key '{}' declared in mok-gpui.toml [media.files] was not found on disk.\n\
                 Searched locations:\n\
                   - {}\n\
                   - {}\n\
                 Run `cargo mok add` for the component that needs it, or fix the path.",
                entry.path,
                key,
                in_assets.display(),
                in_root.display()
            );
        };
        println!("cargo:rerun-if-changed={}", resolved.display());
        resolved_files.push((entry.path.clone(), resolved));
    }

    let mut code = String::new();
    code.push_str("// Generated by build_support/media.rs. Do not edit.\n\n");

    // 1. Image Enum
    code.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    if resolved_images.is_empty() {
        code.push_str("pub enum Image {}\n\n");
    } else {
        code.push_str("pub enum Image {\n");
        for (variant, rel_path, _, _) in &resolved_images {
            code.push_str(&format!("    /// Path: `{}`\n", rel_path));
            code.push_str(&format!("    {},\n", variant));
        }
        code.push_str("}\n\n");
    }

    code.push_str("pub type ImageAsset = Image;\n\n");

    code.push_str("impl Image {\n");
    if resolved_images.is_empty() {
        code.push_str("    pub const fn bytes(&self) -> &'static [u8] { match *self {} }\n");
        code.push_str("    pub const fn path(&self) -> &'static str { match *self {} }\n");
        code.push_str("    pub const fn mime_type(&self) -> &'static str { match *self {} }\n");
    } else {
        code.push_str("    pub const fn bytes(&self) -> &'static [u8] {\n");
        code.push_str("        match self {\n");
        for (variant, _, abs_path, _) in &resolved_images {
            let normalized = abs_path.to_string_lossy().replace('\\', "/");
            code.push_str(&format!(
                "            Self::{} => include_bytes!(r\"{}\"),\n",
                variant, normalized
            ));
        }
        code.push_str("        }\n");
        code.push_str("    }\n\n");

        code.push_str("    pub const fn path(&self) -> &'static str {\n");
        code.push_str("        match self {\n");
        for (variant, rel_path, _, _) in &resolved_images {
            code.push_str(&format!(
                "            Self::{} => \"{}\",\n",
                variant, rel_path
            ));
        }
        code.push_str("        }\n");
        code.push_str("    }\n\n");

        code.push_str("    pub const fn mime_type(&self) -> &'static str {\n");
        code.push_str("        match self {\n");
        for (variant, _, _, mime) in &resolved_images {
            code.push_str(&format!("            Self::{} => \"{}\",\n", variant, mime));
        }
        code.push_str("        }\n");
        code.push_str("    }\n");
    }
    code.push_str("}\n\n");

    // Implement Into<ImageSource> and Into<SharedString> for Image
    code.push_str("impl From<Image> for gpui::SharedString {\n");
    code.push_str("    fn from(asset: Image) -> Self {\n");
    code.push_str("        gpui::SharedString::from(asset.path())\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str("impl From<Image> for gpui::ImageSource {\n");
    code.push_str("    fn from(asset: Image) -> Self {\n");
    code.push_str("        gpui::ImageSource::from(asset.path())\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // 2. Video Enum
    code.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    if resolved_videos.is_empty() {
        code.push_str("pub enum Video {}\n\n");
    } else {
        code.push_str("pub enum Video {\n");
        for (variant, rel_path, _, _) in &resolved_videos {
            code.push_str(&format!("    /// Path: `{}`\n", rel_path));
            code.push_str(&format!("    {},\n", variant));
        }
        code.push_str("}\n\n");
    }

    code.push_str("pub type VideoAsset = Video;\n\n");

    code.push_str("impl Video {\n");
    if resolved_videos.is_empty() {
        code.push_str("    pub const fn bytes(&self) -> &'static [u8] { match *self {} }\n");
        code.push_str("    pub const fn path(&self) -> &'static str { match *self {} }\n");
    } else {
        code.push_str("    pub const fn bytes(&self) -> &'static [u8] {\n");
        code.push_str("        match self {\n");
        for (variant, _, abs_path, embed) in &resolved_videos {
            if *embed {
                let normalized = abs_path.to_string_lossy().replace('\\', "/");
                code.push_str(&format!(
                    "            Self::{} => include_bytes!(r\"{}\"),\n",
                    variant, normalized
                ));
            } else {
                code.push_str(&format!("            Self::{} => &[],\n", variant));
            }
        }
        code.push_str("        }\n");
        code.push_str("    }\n\n");

        code.push_str("    pub const fn path(&self) -> &'static str {\n");
        code.push_str("        match self {\n");
        for (variant, rel_path, _, _) in &resolved_videos {
            code.push_str(&format!(
                "            Self::{} => \"{}\",\n",
                variant, rel_path
            ));
        }
        code.push_str("        }\n");
        code.push_str("    }\n");
    }
    code.push_str("}\n\n");

    // 3. embedded_media lookup
    code.push_str("pub fn embedded_media(path: &str) -> Option<&'static [u8]> {\n");
    code.push_str("    match path {\n");
    for (_, rel_path, abs_path, _) in &resolved_images {
        let normalized = abs_path.to_string_lossy().replace('\\', "/");
        code.push_str(&format!(
            "        \"{}\" => Some(include_bytes!(r\"{}\")),\n",
            rel_path, normalized
        ));
    }
    for (_, rel_path, abs_path, embed) in &resolved_videos {
        if *embed {
            let normalized = abs_path.to_string_lossy().replace('\\', "/");
            code.push_str(&format!(
                "        \"{}\" => Some(include_bytes!(r\"{}\")),\n",
                rel_path, normalized
            ));
        }
    }
    for (rel_path, abs_path) in &resolved_files {
        let normalized = abs_path.to_string_lossy().replace('\\', "/");
        code.push_str(&format!(
            "        \"{}\" => Some(include_bytes!(r\"{}\")),\n",
            rel_path, normalized
        ));
    }
    code.push_str("        _ => None,\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    let mut names: Vec<&str> = resolved_images.iter().map(|i| i.1.as_str()).collect();
    names.extend(resolved_videos.iter().filter(|v| v.3).map(|v| v.1.as_str()));
    names.extend(resolved_files.iter().map(|f| f.0.as_str()));
    code.push_str("/// Every path `embedded_media` answers.\n");
    code.push_str("pub const MEDIA_NAMES: &[&str] = &[\n");
    for name in names {
        code.push_str(&format!("    \"{name}\",\n"));
    }
    code.push_str("];\n");

    let dest = out_dir.join("mok_media.rs");
    fs::write(dest, code).expect("failed to write mok_media.rs");
}
