use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    for part in name.split(['-', '_']) {
        if part.is_empty() {
            continue;
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
    let re =
        regex::Regex::new(r"(?:LucideIcon|HeroIcon|HugeIcon|LucideIcon)::([A-Za-z0-9_]+)").unwrap();

    for pattern in sources {
        let clean_path = pattern
            .replace("/**/*.rs", "")
            .replace("/*.rs", "")
            .replace("**", "");
        let target_dir = root_dir.join(&clean_path);
        if target_dir.exists() {
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

pub fn generate_icons(
    icons_dir: &Path,
    sources: &[String],
    root_dir: &Path,
    out_dir: &Path,
    mode: &str,
    minify: bool,
) {
    let mut lucide_icons = Vec::new();
    if icons_dir.exists()
        && let Ok(entries) = fs::read_dir(icons_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("svg")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                let variant = to_pascal_case(stem);
                lucide_icons.push((variant, stem.to_string(), path));
            }
        }
    }

    lucide_icons.sort_by(|a, b| a.0.cmp(&b.0));

    let used_icons = scan_used_icons(sources, root_dir);

    let mut code = String::new();
    code.push_str("// Auto-generated icons from ely-gpui.toml. Do not edit.\n\n");

    // 1. LucideIcon enum
    code.push_str("/// Lucide icons bundled with Ely.\n");
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    code.push_str("pub enum LucideIcon {\n");
    for (variant, stem, _) in &lucide_icons {
        code.push_str(&format!("    /// `{stem}.svg`\n"));
        code.push_str(&format!("    {variant},\n"));
    }
    code.push_str("}\n\n");

    // LucideIcon methods
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("impl LucideIcon {\n");
    code.push_str("    pub const ALL: &'static [LucideIcon] = &[\n");
    for (variant, _, _) in &lucide_icons {
        code.push_str(&format!("        LucideIcon::{variant},\n"));
    }
    code.push_str("    ];\n\n");

    code.push_str("    pub fn path(self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for (variant, stem, _) in &lucide_icons {
        code.push_str(&format!(
            "            LucideIcon::{variant} => \"icons/{stem}.svg\",\n"
        ));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    code.push_str("    pub fn name(self) -> &'static str {\n");
    code.push_str("        match self {\n");
    for (variant, stem, _) in &lucide_icons {
        code.push_str(&format!(
            "            LucideIcon::{variant} => {:?},\n",
            stem
        ));
    }
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // 2. IconAsset trait
    code.push_str("pub trait IconAsset: Copy + Send + Sync + 'static {\n");
    code.push_str("    fn path(self) -> &'static str;\n");
    code.push_str("    fn name(self) -> &'static str;\n");
    code.push_str("}\n\n");

    code.push_str("impl IconAsset for LucideIcon {\n");
    code.push_str("    fn path(self) -> &'static str { self.path() }\n");
    code.push_str("    fn name(self) -> &'static str { self.name() }\n");
    code.push_str("}\n\n");

    // 3. HeroIcon and HugeIcon provider enums
    let hero_dir = icons_dir.join("heroicons");
    let mut hero_icons = Vec::new();
    if hero_dir.exists()
        && let Ok(entries) = fs::read_dir(&hero_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("svg")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                hero_icons.push((to_pascal_case(stem), stem.to_string(), path));
            }
        }
    }
    hero_icons.sort_by(|a, b| a.0.cmp(&b.0));

    code.push_str("/// Heroicons pack.\n");
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    if hero_icons.is_empty() {
        code.push_str("pub enum HeroIcon {\n    Sparkles,\n    Search,\n    Check,\n    ChevronRight,\n    Plus,\n    XMark,\n}\n\n");
        code.push_str("#[allow(dead_code)]\nimpl IconAsset for HeroIcon {\n");
        code.push_str("    fn path(self) -> &'static str {\n");
        code.push_str("        match self {\n");
        code.push_str("            HeroIcon::Sparkles => \"icons/sparkles.svg\",\n");
        code.push_str("            HeroIcon::Search => \"icons/search.svg\",\n");
        code.push_str("            HeroIcon::Check => \"icons/check.svg\",\n");
        code.push_str("            HeroIcon::ChevronRight => \"icons/chevron-right.svg\",\n");
        code.push_str("            HeroIcon::Plus => \"icons/plus.svg\",\n");
        code.push_str("            HeroIcon::XMark => \"icons/x-mark.svg\",\n");
        code.push_str("        }\n    }\n");
        code.push_str("    fn name(self) -> &'static str {\n");
        code.push_str("        match self {\n");
        code.push_str("            HeroIcon::Sparkles => \"sparkles\",\n");
        code.push_str("            HeroIcon::Search => \"search\",\n");
        code.push_str("            HeroIcon::Check => \"check\",\n");
        code.push_str("            HeroIcon::ChevronRight => \"chevron-right\",\n");
        code.push_str("            HeroIcon::Plus => \"plus\",\n");
        code.push_str("            HeroIcon::XMark => \"x-mark\",\n");
        code.push_str("        }\n    }\n}\n\n");
    } else {
        code.push_str("pub enum HeroIcon {\n");
        for (variant, stem, _) in &hero_icons {
            code.push_str(&format!("    /// `{stem}.svg`\n    {variant},\n"));
        }
        code.push_str("}\n\n#[allow(dead_code)]\nimpl IconAsset for HeroIcon {\n");
        code.push_str("    fn path(self) -> &'static str {\n        match self {\n");
        for (variant, stem, _) in &hero_icons {
            code.push_str(&format!(
                "            HeroIcon::{variant} => \"icons/heroicons/{stem}.svg\",\n"
            ));
        }
        code.push_str("        }\n    }\n");
        code.push_str("    fn name(self) -> &'static str {\n        match self {\n");
        for (variant, stem, _) in &hero_icons {
            code.push_str(&format!("            HeroIcon::{variant} => {:?},\n", stem));
        }
        code.push_str("        }\n    }\n}\n\n");
    }

    let huge_dir = icons_dir.join("hugeicons");
    let mut huge_icons = Vec::new();
    if huge_dir.exists()
        && let Ok(entries) = fs::read_dir(&huge_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("svg")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                huge_icons.push((to_pascal_case(stem), stem.to_string(), path));
            }
        }
    }
    huge_icons.sort_by(|a, b| a.0.cmp(&b.0));

    code.push_str("/// Hugeicons pack.\n");
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n");
    if huge_icons.is_empty() {
        code.push_str("pub enum HugeIcon {\n    AiChat01,\n    CodeBrowser,\n    CursorPointer01,\n    Folder01,\n}\n\n");
        code.push_str("#[allow(dead_code)]\nimpl IconAsset for HugeIcon {\n");
        code.push_str("    fn path(self) -> &'static str {\n");
        code.push_str("        match self {\n");
        code.push_str("            HugeIcon::AiChat01 => \"icons/bot.svg\",\n");
        code.push_str("            HugeIcon::CodeBrowser => \"icons/code.svg\",\n");
        code.push_str("            HugeIcon::CursorPointer01 => \"icons/mouse-pointer.svg\",\n");
        code.push_str("            HugeIcon::Folder01 => \"icons/folder.svg\",\n");
        code.push_str("        }\n    }\n");
        code.push_str("    fn name(self) -> &'static str {\n");
        code.push_str("        match self {\n");
        code.push_str("            HugeIcon::AiChat01 => \"bot\",\n");
        code.push_str("            HugeIcon::CodeBrowser => \"code\",\n");
        code.push_str("            HugeIcon::CursorPointer01 => \"mouse-pointer\",\n");
        code.push_str("            HugeIcon::Folder01 => \"folder\",\n");
        code.push_str("        }\n    }\n}\n\n");
    } else {
        code.push_str("pub enum HugeIcon {\n");
        for (variant, stem, _) in &huge_icons {
            code.push_str(&format!("    /// `{stem}.svg`\n    {variant},\n"));
        }
        code.push_str("}\n\n#[allow(dead_code)]\nimpl IconAsset for HugeIcon {\n");
        code.push_str("    fn path(self) -> &'static str {\n        match self {\n");
        for (variant, stem, _) in &huge_icons {
            code.push_str(&format!(
                "            HugeIcon::{variant} => \"icons/hugeicons/{stem}.svg\",\n"
            ));
        }
        code.push_str("        }\n    }\n");
        code.push_str("    fn name(self) -> &'static str {\n        match self {\n");
        for (variant, stem, _) in &huge_icons {
            code.push_str(&format!("            HugeIcon::{variant} => {:?},\n", stem));
        }
        code.push_str("        }\n    }\n}\n\n");
    }

    // 4. Pruned and minified embedded icons table
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("pub const EMBEDDED_ICONS: &[(&str, &[u8])] = &[\n");
    let embed_all = mode == "all";
    for (variant, stem, path) in &lucide_icons {
        if (embed_all || used_icons.contains(variant))
            && let Ok(content) = fs::read_to_string(path)
        {
            let final_svg = if minify {
                minify_svg(&content)
            } else {
                content
            };
            code.push_str(&format!(
                "    (\"icons/{stem}.svg\", {:?}.as_bytes()),\n",
                final_svg
            ));
        }
    }
    for (variant, stem, path) in &hero_icons {
        if (embed_all || used_icons.contains(variant))
            && let Ok(content) = fs::read_to_string(path)
        {
            let final_svg = if minify {
                minify_svg(&content)
            } else {
                content
            };
            code.push_str(&format!(
                "    (\"icons/heroicons/{stem}.svg\", {:?}.as_bytes()),\n",
                final_svg
            ));
        }
    }
    for (variant, stem, path) in &huge_icons {
        if (embed_all || used_icons.contains(variant))
            && let Ok(content) = fs::read_to_string(path)
        {
            let final_svg = if minify {
                minify_svg(&content)
            } else {
                content
            };
            code.push_str(&format!(
                "    (\"icons/hugeicons/{stem}.svg\", {:?}.as_bytes()),\n",
                final_svg
            ));
        }
    }
    code.push_str("];\n");

    let dest = out_dir.join("ely_icons.rs");
    fs::write(&dest, code).expect("failed to write ely_icons.rs");
}
