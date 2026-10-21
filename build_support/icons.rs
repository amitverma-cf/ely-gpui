use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    for part in name.split(['-', '_']) {
        if part.is_empty() {
            continue;
        }
        // `arrow-down-0-1` and `arrow-down-01` are two icons: a digit after a digit takes an underscore.
        if part.starts_with(|c: char| c.is_ascii_digit())
            && result.ends_with(|c: char| c.is_ascii_digit())
        {
            result.push('_');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.extend(chars);
        }
    }
    result
}

pub fn scan_used_icons(sources: &[String], root_dir: &Path) -> HashSet<String> {
    let mut used = HashSet::new();
    let re = regex::Regex::new(r"(?:LucideIcon|HeroIcon|HugeIcon)::([A-Za-z0-9_]+)").unwrap();

    for pattern in sources {
        let clean_path = pattern
            .replace("/**/*.rs", "")
            .replace("/*.rs", "")
            .replace("**", "");
        let target_dir = root_dir.join(&clean_path);
        if target_dir.exists() {
            println!("cargo:rerun-if-changed={}", target_dir.display());
            for entry in walkdir::WalkDir::new(&target_dir).into_iter().flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
                    && let Ok(content) = fs::read_to_string(entry.path())
                {
                    for cap in re.captures_iter(&content) {
                        used.insert(cap[1].to_string());
                    }
                }
            }
        }
    }

    used
}

pub fn minify_svg(raw: &str) -> String {
    let mut s = raw.trim();
    if let Some(idx) = s.find("<svg") {
        s = &s[idx..];
    }
    let mut cleaned = s.to_string();
    while let Some(start) = cleaned.find("<!--") {
        if let Some(end) = cleaned[start..].find("-->") {
            cleaned.replace_range(start..start + end + 3, "");
        } else {
            break;
        }
    }
    let ws_re = regex::Regex::new(r"\s+").unwrap();
    let tag_re = regex::Regex::new(r">\s+<").unwrap();
    let collapsed = ws_re.replace_all(&cleaned, " ");
    tag_re.replace_all(&collapsed, "><").trim().to_string()
}

/// One icon pack on disk: its enum, where its SVGs sit, and the asset path they answer to.
struct Pack {
    enum_name: &'static str,
    doc: &'static str,
    dir: PathBuf,
    prefix: &'static str,
}

/// `(variant, file stem, path)` for each SVG in the pack, by variant.
fn read_pack(dir: &Path) -> Vec<(String, String, PathBuf)> {
    let mut icons = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("svg")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                icons.push((to_pascal_case(stem), stem.to_string(), path));
            }
        }
    }
    icons.sort_by(|a, b| a.0.cmp(&b.0));
    icons
}

/// The enum, its `ALL`, `path` and `name`, and its `IconAsset`; an uninstalled pack has no variants.
fn provider(code: &mut String, pack: &Pack, icons: &[(String, String, PathBuf)]) {
    let name = pack.enum_name;
    let variants = |f: &dyn Fn(&str, &str) -> String| -> String {
        icons.iter().map(|(v, s, _)| f(v, s)).collect()
    };
    code.push_str(&format!("/// {}\n", pack.doc));
    code.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    code.push_str(&format!("pub enum {name} {{\n"));
    code.push_str(&variants(&|v, s| format!("    /// `{s}.svg`\n    {v},\n")));
    code.push_str("}\n\n");
    code.push_str(&format!("impl {name} {{\n"));
    code.push_str(&format!("    pub const ALL: &'static [{name}] = &[\n"));
    code.push_str(&variants(&|v, _| format!("        {name}::{v},\n")));
    code.push_str("    ];\n\n");
    code.push_str("    pub fn path(self) -> &'static str {\n        match self {\n");
    code.push_str(&variants(&|v, s| {
        format!("            {name}::{v} => \"{}{s}.svg\",\n", pack.prefix)
    }));
    code.push_str("        }\n    }\n\n");
    code.push_str("    pub fn name(self) -> &'static str {\n        match self {\n");
    code.push_str(&variants(&|v, s| {
        format!("            {name}::{v} => {s:?},\n")
    }));
    code.push_str("        }\n    }\n}\n\n");
    code.push_str(&format!("impl IconAsset for {name} {{\n"));
    code.push_str("    fn path(self) -> &'static str { self.path() }\n");
    code.push_str("    fn name(self) -> &'static str { self.name() }\n");
    code.push_str("}\n\n");
}

pub fn generate_icons(
    icons_dir: &Path,
    sources: &[String],
    root_dir: &Path,
    out_dir: &Path,
    mode: &str,
    minify: bool,
) {
    let packs = [
        Pack {
            enum_name: "LucideIcon",
            doc: "Lucide icons: those the code names are embedded.",
            dir: icons_dir.to_path_buf(),
            prefix: "icons/",
        },
        Pack {
            enum_name: "HeroIcon",
            doc: "Heroicons, when `cargo mok add icons heroicons` has fetched them.",
            dir: icons_dir.join("heroicons"),
            prefix: "icons/heroicons/",
        },
        Pack {
            enum_name: "HugeIcon",
            doc: "Hugeicons, when `cargo mok add icons hugeicons` has fetched them.",
            dir: icons_dir.join("hugeicons"),
            prefix: "icons/hugeicons/",
        },
    ];
    let used = scan_used_icons(sources, root_dir);
    let mut code = String::from("// Auto-generated icons from mok-gpui.toml. Do not edit.\n\n");
    code.push_str("pub trait IconAsset: Copy + Send + Sync + 'static {\n");
    code.push_str("    fn path(self) -> &'static str;\n");
    code.push_str("    fn name(self) -> &'static str;\n");
    code.push_str("}\n\n");

    let mut table = String::new();
    for pack in &packs {
        let icons = read_pack(&pack.dir);
        if icons.is_empty() && pack.enum_name == "LucideIcon" {
            panic!(
                "mok: no Lucide icons in {}. Run `cargo mok init` to download them.",
                pack.dir.display()
            );
        }
        provider(&mut code, pack, &icons);
        for (variant, stem, path) in &icons {
            if mode != "all" && !used.contains(variant) {
                continue;
            }
            let Ok(content) = fs::read_to_string(path) else {
                continue;
            };
            let svg = if minify {
                minify_svg(&content)
            } else {
                content
            };
            table.push_str(&format!(
                "    (\"{}{stem}.svg\", {svg:?}.as_bytes()),\n",
                pack.prefix
            ));
        }
    }
    code.push_str(
        "/// The icons the code uses, minified; the rest of each pack stays out of the binary.\n",
    );
    code.push_str("pub const EMBEDDED_ICONS: &[(&str, &[u8])] = &[\n");
    code.push_str(&table);
    code.push_str("];\n");
    fs::write(out_dir.join("mok_icons.rs"), code).expect("failed to write mok_icons.rs");
}
