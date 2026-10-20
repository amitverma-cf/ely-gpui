//! Fonts and icons come from their upstream releases, pinned by checksum, into the project's `assets/`.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

struct Pin {
    url: &'static str,
    sha256: &'static str,
}

const INTER: Pin = Pin {
    url: "https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip",
    sha256: "9883fdd4a49d4fb66bd8177ba6625ef9a64aa45899767dde3d36aa425756b11e",
};
const JETBRAINS_REGULAR: Pin = Pin {
    url: "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/v2.304/fonts/ttf/JetBrainsMono-Regular.ttf",
    sha256: "a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f",
};
const JETBRAINS_MEDIUM: Pin = Pin {
    url: "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/v2.304/fonts/ttf/JetBrainsMono-Medium.ttf",
    sha256: "31c92d01a8a08528b718a43addf0ad3df0af2ca4b7b3290a452f70f358e14d3d",
};
const JETBRAINS_LICENSE: Pin = Pin {
    url: "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/v2.304/OFL.txt",
    sha256: "30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad",
};
const PLEX_REGULAR: Pin = Pin {
    url: "https://raw.githubusercontent.com/IBM/plex/%40ibm/plex-sans%401.1.0/packages/plex-sans/fonts/complete/ttf/IBMPlexSans-Regular.ttf",
    sha256: "975dcda37d80f038dcd143c22e33ca2d97a0cc5a929aace1c749153b0fe1afa5",
};
const PLEX_LICENSE: Pin = Pin {
    url: "https://raw.githubusercontent.com/IBM/plex/%40ibm/plex-sans%401.1.0/LICENSE.txt",
    sha256: "7e6b2818edbd8f6a01ae80641cc8f16a51080d08fb4e532be3a0b6f74adb07da",
};
const LUCIDE: Pin = Pin {
    url: "https://registry.npmjs.org/lucide-static/-/lucide-static-1.48.0.tgz",
    sha256: "3c2ecda3d25f6a9692d83f8036d9a526f7da584a51af74cd16eda4498c5c33d8",
};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The bytes at `pin.url`, checked against its checksum.
fn get(pin: &Pin) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .args(["-fsSL", pin.url])
        .output()
        .map_err(|e| format!("curl is needed to download {}: {e}", pin.url))?;
    if !out.status.success() {
        return Err(format!(
            "download of {} failed: {}",
            pin.url,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let found = hex(&Sha256::digest(&out.stdout));
    if found != pin.sha256 {
        return Err(format!(
            "{} does not match its pinned checksum (expected {}, got {found})",
            pin.url, pin.sha256
        ));
    }
    Ok(out.stdout)
}

fn put(dest: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(dest, bytes).map_err(|e| format!("{}: {e}", dest.display()))
}

/// Inter, JetBrains Mono and IBM Plex Sans into `fonts/`; what is there already stays.
pub fn fonts(fonts: &Path) -> Result<usize, String> {
    let mut count = 0;
    let mut single = |pin: &Pin, name: &str| -> Result<(), String> {
        let dest = fonts.join(name);
        if dest.is_file() {
            return Ok(());
        }
        put(&dest, &get(pin)?)?;
        count += 1;
        Ok(())
    };
    single(&JETBRAINS_REGULAR, "JetBrainsMono-Regular.ttf")?;
    single(&JETBRAINS_MEDIUM, "JetBrainsMono-Medium.ttf")?;
    single(&JETBRAINS_LICENSE, "JetBrainsMono-LICENSE.txt")?;
    single(&PLEX_REGULAR, "ibm-plex-sans/IBMPlexSans-Regular.ttf")?;
    single(&PLEX_LICENSE, "ibm-plex-sans/IBMPlexSans-LICENSE.txt")?;
    let wanted = [
        ("extras/ttf/Inter-Regular.ttf", "Inter-Regular.ttf"),
        ("extras/ttf/Inter-Medium.ttf", "Inter-Medium.ttf"),
        ("extras/ttf/Inter-SemiBold.ttf", "Inter-SemiBold.ttf"),
        ("extras/ttf/Inter-Italic.ttf", "Inter-Italic.ttf"),
        ("LICENSE.txt", "Inter-LICENSE.txt"),
    ];
    if wanted.iter().any(|(_, name)| !fonts.join(name).is_file()) {
        let zip = get(&INTER)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip))
            .map_err(|e| format!("Inter's archive: {e}"))?;
        for (inside, name) in wanted {
            let mut file = archive
                .by_name(inside)
                .map_err(|e| format!("Inter's archive lacks {inside}: {e}"))?;
            let mut bytes = vec![];
            file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            put(&fonts.join(name), &bytes)?;
            count += 1;
        }
    }
    Ok(count)
}

/// Every Lucide icon, flat, with its license, into `icons/`.
pub fn lucide(icons: &Path) -> Result<usize, String> {
    let tgz = get(&LUCIDE)?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tgz.as_slice()));
    let mut count = 0;
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        let name = path.to_string_lossy().replace('\\', "/");
        let dest = if let Some(file) = name.strip_prefix("package/icons/") {
            file.ends_with(".svg").then(|| icons.join(file))
        } else if name == "package/LICENSE" {
            Some(icons.join("LICENSE"))
        } else {
            None
        };
        let Some(dest) = dest else { continue };
        let mut bytes = vec![];
        entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        put(&dest, &bytes)?;
        count += 1;
    }
    if count == 0 {
        return Err("Lucide's package held no icons".into());
    }
    Ok(count)
}
