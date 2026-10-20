//! What the task notes say about each component.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use regex::Regex;

pub fn key(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

pub fn key_ident(text: &str) -> String {
    text.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

pub fn snake(text: &str) -> String {
    let mut out = String::new();
    for (i, c) in text.replace(' ', "").chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c);
    }
    out.to_lowercase().replace('-', "_")
}

pub struct Task {
    /// The prose after a dash.
    pub desc: String,
    /// What follows an arrow: where the component lives, in the notes' words.
    pub arrow: String,
    pub target: String,
    /// The other names on the same line, which stand for the same component.
    pub group: Vec<String>,
}

/// What the task notes say about each component, by its alphanumeric lowercase name.
pub fn tasks(repo: &Path) -> Result<BTreeMap<String, Task>, String> {
    let paren = Regex::new(r"\s*\([^)]*\)").unwrap();
    let target = Regex::new(r"^`?([\w:]+)`?").unwrap();
    let mut out = BTreeMap::new();
    let mut files: Vec<_> = fs::read_dir(repo.join("tasks"))
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ch") && n.ends_with(".md"))
        })
        .collect();
    files.sort();
    for file in files {
        let text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        for line in text.lines() {
            let line = line.trim();
            let Some(rest) = line
                .strip_prefix("- [x] ")
                .or_else(|| line.strip_prefix("- [ ] "))
            else {
                continue;
            };
            let dash = rest.find(" — ");
            let arrow = rest.find(" → ");
            let split = match (dash, arrow) {
                (Some(d), Some(a)) => d.min(a),
                (Some(d), None) => d,
                (None, Some(a)) => a,
                (None, None) => rest.len(),
            };
            let names = &rest[..split];
            let desc = dash
                .map_or("", |d| rest[d + " — ".len()..].trim())
                .replace('`', "");
            let after = arrow
                .map_or("", |a| rest[a + " → ".len()..].trim())
                .replace('`', "");
            let tgt = arrow.map_or("", |a| {
                target
                    .captures(&rest[a + " → ".len()..])
                    .map_or("", |c| c.get(1).unwrap().as_str())
            });
            let group: Vec<String> = names
                .split(" / ")
                .map(|n| key_ident(paren.replace_all(n, "").trim()))
                .filter(|n| !n.is_empty())
                .collect();
            for name in names.split(" / ") {
                let clean = paren.replace_all(name, "").trim().to_string();
                if !clean.is_empty() {
                    out.entry(key(&clean)).or_insert(Task {
                        desc: desc.clone(),
                        arrow: after.clone(),
                        target: tgt.to_string(),
                        group: group.clone(),
                    });
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_become_snake_case_and_keys() {
        assert_eq!(snake("FocusScope"), "focus_scope");
        assert_eq!(snake("Tooltip Trigger"), "tooltip_trigger");
        assert_eq!(key("Block types"), "blocktypes");
        assert_eq!(key_ident("3D Model"), "3DModel");
    }

    #[test]
    fn a_note_gives_its_prose_its_target_and_its_siblings() {
        let dir = std::env::temp_dir().join(format!("ely-notes-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("tasks")).unwrap();
        std::fs::write(
            dir.join("tasks/ch01.md"),
            "- [x] Divider / Separator\n- [x] CursorRenderer — block, line → `CodeEditor::cursor_shape`\n- [ ] Ripple → `motion::Ripple`\n",
        )
        .unwrap();
        let notes = tasks(&dir).unwrap();
        assert_eq!(notes["separator"].group, ["Divider", "Separator"]);
        assert_eq!(
            notes["cursorrenderer"].desc,
            "block, line → CodeEditor::cursor_shape"
        );
        assert_eq!(notes["cursorrenderer"].target, "CodeEditor::cursor_shape");
        assert_eq!(notes["ripple"].target, "motion::Ripple");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
