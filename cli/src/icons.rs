use std::fs;
use std::path::Path;
use std::process::Command;

use crate::fetch;

/// Puts the pack's SVGs in `dest_dir`; every failure is an error.
pub fn download_icon_pack(pack: &str, dest_dir: &Path) -> Result<usize, String> {
    fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create icon directory: {e}"))?;
    match pack.to_ascii_lowercase().as_str() {
        "lucide" => fetch::lucide(dest_dir),
        "heroicons" => from_git(
            "https://github.com/tailwindlabs/heroicons.git",
            &["src", "24", "outline"],
            dest_dir,
        ),
        "hugeicons" => from_git(
            "https://github.com/hugeicons/hugeicons-free-icons.git",
            &["icons", "stroke"],
            dest_dir,
        ),
        other => Err(format!(
            "Unknown icon pack '{other}'. Supported: lucide, heroicons, hugeicons"
        )),
    }
}

fn from_git(url: &str, inside: &[&str], dest: &Path) -> Result<usize, String> {
    let temp = std::env::temp_dir().join(format!("ely_icons_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--quiet", url])
        .arg(&temp)
        .status()
        .map_err(|e| format!("git is needed to fetch {url}: {e}"))?;
    if !status.success() {
        return Err(format!("git clone {url} failed"));
    }
    let src = inside.iter().fold(temp.clone(), |p, part| p.join(part));
    let result = copy_svgs(&src, dest);
    let _ = fs::remove_dir_all(&temp);
    result
}

fn copy_svgs(src: &Path, dest: &Path) -> Result<usize, String> {
    let entries =
        fs::read_dir(src).map_err(|e| format!("{} is not readable: {e}", src.display()))?;
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "svg")
            && let Some(name) = path.file_name()
        {
            fs::copy(&path, dest.join(name)).map_err(|e| format!("{}: {e}", path.display()))?;
            count += 1;
        }
    }
    if count == 0 {
        return Err(format!("{} held no SVG icons", src.display()));
    }
    Ok(count)
}
