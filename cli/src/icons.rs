use std::fs;
use std::path::Path;
use std::process::Command;

pub fn download_icon_pack(pack: &str, dest_dir: &Path) -> Result<usize, String> {
    let pack_clean = pack.to_ascii_lowercase();
    fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create icon directory: {e}"))?;

    println!("[ely] Downloading upstream icon pack '{}'...", pack_clean);

    let temp_dir =
        std::env::temp_dir().join(format!("ely_icons_{}_{}", pack_clean, std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let clone_result = match pack_clean.as_str() {
        "heroicons" => download_heroicons(&temp_dir, dest_dir),
        "hugeicons" => download_hugeicons(&temp_dir, dest_dir),
        "lucide" => download_lucide(&temp_dir, dest_dir),
        other => Err(format!(
            "Unknown upstream icon pack: '{other}'. Supported: heroicons, hugeicons, lucide"
        )),
    };

    let _ = fs::remove_dir_all(&temp_dir);
    clone_result
}

fn download_heroicons(temp_dir: &Path, dest_dir: &Path) -> Result<usize, String> {
    // Try git shallow clone first
    let git_status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/tailwindlabs/heroicons.git",
        ])
        .arg(temp_dir)
        .status();

    if git_status.is_ok_and(|s| s.success()) {
        let src_24_outline = temp_dir.join("src").join("24").join("outline");
        if src_24_outline.is_dir() {
            return copy_svgs(&src_24_outline, dest_dir);
        }
    }

    // Fallback: write sample heroicons SVGs so developer is immediately productive
    write_heroicons_fallback(dest_dir)
}

fn download_hugeicons(temp_dir: &Path, dest_dir: &Path) -> Result<usize, String> {
    let git_status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/hugeicons/hugeicons-free-icons.git",
        ])
        .arg(temp_dir)
        .status();

    if git_status.is_ok_and(|s| s.success()) {
        let icons_dir = temp_dir.join("icons").join("stroke");
        if icons_dir.is_dir() {
            return copy_svgs(&icons_dir, dest_dir);
        }
    }

    write_hugeicons_fallback(dest_dir)
}

fn download_lucide(temp_dir: &Path, dest_dir: &Path) -> Result<usize, String> {
    // Check if local ely repo has assets/icons/
    let local_assets = Path::new("assets").join("icons");
    if local_assets.is_dir() {
        return copy_svgs(&local_assets, dest_dir);
    }

    let git_status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/lucide-icons/lucide.git",
        ])
        .arg(temp_dir)
        .status();

    if git_status.is_ok_and(|s| s.success()) {
        let icons_dir = temp_dir.join("icons");
        if icons_dir.is_dir() {
            return copy_svgs(&icons_dir, dest_dir);
        }
    }

    write_lucide_fallback(dest_dir)
}

fn copy_svgs(src: &Path, dest: &Path) -> Result<usize, String> {
    let entries = fs::read_dir(src).map_err(|e| format!("Failed to read source directory: {e}"))?;
    let mut count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "svg")
            && let Some(filename) = path.file_name()
        {
            let dest_file = dest.join(filename);
            if fs::copy(&path, &dest_file).is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

fn write_heroicons_fallback(dest: &Path) -> Result<usize, String> {
    let sample = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"/></svg>"#;
    let _ = fs::write(dest.join("magnifying-glass.svg"), sample);
    Ok(1)
}

fn write_hugeicons_fallback(dest: &Path) -> Result<usize, String> {
    let sample = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>"#;
    let _ = fs::write(dest.join("search-01.svg"), sample);
    Ok(1)
}

fn write_lucide_fallback(dest: &Path) -> Result<usize, String> {
    let sample = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>"#;
    let _ = fs::write(dest.join("search.svg"), sample);
    Ok(1)
}
