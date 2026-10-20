use std::fs;
use std::path::Path;

use serde::Deserialize;

const EMBEDDED: &str = include_str!("../../registry.json");

#[derive(Debug, Deserialize, Clone)]
pub struct RegistryComponent {
    pub name: String,
    pub display_name: String,
    pub chapter: usize,
    pub export_type: String,
    pub description: String,
    /// `component`, `feature`, `core`, `native`, `blocked`, `host` or `unmapped`.
    pub status: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub import: Option<String>,
    #[serde(default)]
    pub files: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RegistryChapter {
    pub number: usize,
    pub name: String,
    pub title: String,
    pub summary: String,
    pub component_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct Registry {
    pub total_components: usize,
    pub total_chapters: usize,
    pub chapters: Vec<RegistryChapter>,
    pub components: Vec<RegistryComponent>,
}

/// The registry in a checkout when one is given, else the one this binary was built with.
pub fn load_registry(source: Option<&Path>) -> Result<Registry, String> {
    let text = match source {
        Some(dir) => fs::read_to_string(dir.join("registry.json"))
            .map_err(|e| format!("{}: {e}", dir.join("registry.json").display()))?,
        None => EMBEDDED.to_string(),
    };
    serde_json::from_str(&text).map_err(|e| format!("registry.json: {e}"))
}

fn tag(c: &RegistryComponent) -> String {
    match c.status.as_str() {
        "component" => c.files.map_or_else(String::new, |n| format!("{n} files")),
        "feature" => format!("part of {}", c.export_type),
        "core" => "in ely-gpui".into(),
        "native" => "native to gpui".into(),
        "blocked" => "blocked in gpui".into(),
        "host" => "host provides".into(),
        _ => "no code yet".into(),
    }
}

fn line(c: &RegistryComponent) {
    println!("  {:<28} {:<16} {}", c.name, tag(c), c.description);
}

pub fn list_components(filter: Option<&str>, as_json: bool) -> Result<(), String> {
    if as_json {
        println!("{EMBEDDED}");
        return Ok(());
    }
    let registry = load_registry(None)?;
    let total = registry.total_components;
    match filter {
        None => {
            println!("Ely GPUI Component Registry");
            println!(
                "Total: {total} components across {} chapters\n",
                registry.total_chapters
            );
            println!("Chapters:");
            for ch in &registry.chapters {
                println!(
                    "  {:02}. {:<24} ({} components)",
                    ch.number, ch.title, ch.component_count
                );
            }
            println!("\nUsage:");
            println!("  cargo ely list <chapter>        List a chapter (e.g. 'forms', 'charts')");
            println!("  cargo ely list search <query>   Search all {total} components");
            println!("  cargo ely list --all            List all {total}, grouped by chapter");
            println!("  cargo ely add <component>       Copy a component into src/ui as source\n");
        }
        Some("--all") => {
            println!("Ely GPUI Component Registry (all {total} components)\n");
            for ch in &registry.chapters {
                println!(
                    "== {:02}. {} ({}) ==",
                    ch.number, ch.title, ch.component_count
                );
                registry
                    .components
                    .iter()
                    .filter(|c| c.chapter == ch.number)
                    .for_each(line);
                println!();
            }
        }
        Some(query) if query.starts_with("search ") || query.starts_with("find ") => {
            let term = query
                .trim_start_matches("search ")
                .trim_start_matches("find ")
                .trim()
                .to_lowercase();
            search(&registry, &term);
        }
        Some(target) => {
            let clean = target.trim().to_lowercase();
            let chapter = registry.chapters.iter().find(|ch| {
                ch.name == clean
                    || ch.title.to_lowercase() == clean
                    || format!("{:02}", ch.number) == clean
                    || ch.number.to_string() == clean
            });
            match chapter {
                Some(ch) => {
                    println!(
                        "== {:02}. {} ({}) ==",
                        ch.number, ch.title, ch.component_count
                    );
                    println!("{}\n", ch.summary);
                    registry
                        .components
                        .iter()
                        .filter(|c| c.chapter == ch.number)
                        .for_each(line);
                    println!("\nCopy any of these with: cargo ely add <component>\n");
                }
                None => search(&registry, &clean),
            }
        }
    }
    Ok(())
}

fn search(registry: &Registry, term: &str) {
    let compact = term.replace(['-', ' '], "_");
    let hits: Vec<&RegistryComponent> = registry
        .components
        .iter()
        .filter(|c| {
            c.name.contains(&compact)
                || c.display_name.to_lowercase().contains(term)
                || c.description.to_lowercase().contains(term)
        })
        .collect();
    println!("Search results for '{term}' ({} found):\n", hits.len());
    hits.into_iter().for_each(line);
    println!("\nCopy with: cargo ely add <component>\n");
}

fn find<'a>(registry: &'a Registry, name: &str) -> Option<&'a RegistryComponent> {
    let clean = name.to_ascii_lowercase().replace('-', "_");
    let compact = clean.replace('_', "");
    registry.components.iter().find(|c| {
        c.name == clean
            || c.name.replace('_', "") == compact
            || c.display_name.to_lowercase().replace(' ', "") == compact
            || c.export_type.to_lowercase() == compact
    })
}

/// Copies each named component, with what it needs, into the project.
pub fn scaffold_components(
    names: &[String],
    project_root: &Path,
    opts: &crate::install::Options,
) -> Result<(), String> {
    let source = crate::install::source_root(opts.from.as_deref())?;
    let registry = load_registry(Some(&source))?;
    let mut units = vec![];
    for name in names {
        let entry = find(&registry, name).ok_or_else(|| {
            format!("Unknown component '{name}'. Try 'cargo ely list search {name}'.")
        })?;
        match (entry.status.as_str(), &entry.unit) {
            ("component" | "feature", Some(unit)) => {
                if entry.status == "feature" {
                    println!(
                        "[ely] '{}' is part of {}; copying that.",
                        entry.display_name, entry.export_type
                    );
                }
                units.push(unit.clone());
            }
            ("core", _) => println!(
                "[ely] '{}' ships in the ely-gpui crate: use {}",
                entry.display_name,
                entry.import.as_deref().unwrap_or(&entry.export_type)
            ),
            ("native" | "blocked" | "host", _) => println!(
                "[ely] '{}' has nothing to copy ({}): {}",
                entry.display_name, entry.status, entry.description
            ),
            _ => {
                return Err(format!(
                    "'{}' is not a copyable component: {}",
                    entry.display_name, entry.description
                ));
            }
        }
    }
    if units.is_empty() {
        return Ok(());
    }
    let opts = crate::install::Options {
        from: Some(source),
        dir: opts.dir.clone(),
        force: opts.force,
    };
    crate::install::install(project_root, &units, &opts)?;
    println!("Import from `crate::ui::...`; the code is yours to edit.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn the_registry_counts_agree_and_every_copyable_entry_has_its_files() {
        let registry = load_registry(None).unwrap();
        let units: serde_json::Value = serde_json::from_str(EMBEDDED).unwrap();
        let units = units["units"].as_object().unwrap();
        assert_eq!(registry.total_components, registry.components.len());
        let listed: usize = registry.chapters.iter().map(|c| c.component_count).sum();
        assert_eq!(
            listed,
            registry.components.len(),
            "chapters list every entry"
        );
        let names: BTreeSet<_> = registry.components.iter().map(|c| &c.name).collect();
        assert_eq!(names.len(), registry.components.len(), "names are unique");
        for c in &registry.components {
            if c.status == "component" || c.status == "feature" {
                let unit = c
                    .unit
                    .as_ref()
                    .unwrap_or_else(|| panic!("{} has no unit", c.name));
                assert!(
                    units.contains_key(unit),
                    "{} names {unit}, not in the graph",
                    c.name
                );
            }
        }
    }
}
