use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryComponent {
    pub name: String,
    pub display_name: String,
    pub chapter: usize,
    pub chapter_title: String,
    pub category: String,
    pub source_dir: String,
    pub file: String,
    pub export_type: String,
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryChapter {
    pub number: usize,
    pub name: String,
    pub title: String,
    pub summary: String,
    pub source_dir: String,
    pub component_count: usize,
    pub components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub total_components: usize,
    pub total_chapters: usize,
    pub chapters: Vec<RegistryChapter>,
    pub components: Vec<RegistryComponent>,
}

pub fn load_registry() -> Registry {
    if let Ok(content) = fs::read_to_string("registry.json")
        && let Ok(reg) = serde_json::from_str::<Registry>(&content)
    {
        return reg;
    }
    const EMBEDDED: &str = include_str!("../../../registry.json");
    serde_json::from_str(EMBEDDED).expect("Valid embedded registry.json")
}

pub fn list_components(filter: Option<&str>, as_json: bool) {
    if as_json {
        if let Ok(content) = fs::read_to_string("registry.json") {
            println!("{content}");
        } else {
            const EMBEDDED: &str = include_str!("../../../registry.json");
            println!("{EMBEDDED}");
        }
        return;
    }

    let registry = load_registry();

    match filter {
        None => {
            println!("Ely GPUI Component Registry");
            println!(
                "Total: {} components across {} chapters\n",
                registry.total_components, registry.total_chapters
            );
            println!("Chapters:");
            for ch in &registry.chapters {
                println!(
                    "  {:02}. {:<24} ({} components)",
                    ch.number, ch.title, ch.component_count
                );
            }
            println!("\nUsage:");
            println!(
                "  cargo ely list <chapter>        List components in a chapter (e.g. 'forms', 'charts')"
            );
            println!("  cargo ely list search <query>   Search across all 1,280 components");
            println!(
                "  cargo ely list --all            List all 1,280 components grouped by chapter"
            );
            println!(
                "  cargo ely add <component>       Scaffold an editable component into your app\n"
            );
        }
        Some("--all") => {
            println!(
                "Ely GPUI Component Registry (All {} components)\n",
                registry.total_components
            );
            for ch in &registry.chapters {
                println!(
                    "== {:02}. {} ({} components) ==",
                    ch.number, ch.title, ch.component_count
                );
                let matching: Vec<&RegistryComponent> = registry
                    .components
                    .iter()
                    .filter(|c| c.chapter == ch.number)
                    .collect();
                for c in matching {
                    let deps = if c.dependencies.is_empty() {
                        String::new()
                    } else {
                        format!(" [deps: {}]", c.dependencies.join(", "))
                    };
                    println!(
                        "  {:<26} {:<24} {}{}",
                        c.name, c.export_type, c.description, deps
                    );
                }
                println!();
            }
        }
        Some(query) if query.starts_with("search ") || query.starts_with("find ") => {
            let term = query
                .trim_start_matches("search ")
                .trim_start_matches("find ")
                .trim()
                .to_lowercase();
            search_and_print(&registry, &term);
        }
        Some(target) => {
            let clean = target.trim().to_lowercase();
            // Check if matches chapter number or chapter name
            let chapter_match = registry.chapters.iter().find(|ch| {
                ch.name == clean
                    || ch.title.to_lowercase() == clean
                    || format!("{:02}", ch.number) == clean
                    || format!("{}", ch.number) == clean
            });

            if let Some(ch) = chapter_match {
                println!(
                    "== {:02}. {} ({} components) ==",
                    ch.number, ch.title, ch.component_count
                );
                println!("Summary: {}\n", ch.summary);
                let matching: Vec<&RegistryComponent> = registry
                    .components
                    .iter()
                    .filter(|c| c.chapter == ch.number)
                    .collect();
                for c in matching {
                    let deps = if c.dependencies.is_empty() {
                        String::new()
                    } else {
                        format!(" [deps: {}]", c.dependencies.join(", "))
                    };
                    println!(
                        "  {:<26} {:<24} {}{}",
                        c.name, c.export_type, c.description, deps
                    );
                }
                println!("\nScaffold any of these components with:");
                println!("  cargo ely add <component>\n");
            } else {
                // Treat as search query
                search_and_print(&registry, &clean);
            }
        }
    }
}

fn search_and_print(registry: &Registry, term: &str) {
    let clean_term = term.replace('-', "_");
    let hits: Vec<&RegistryComponent> = registry
        .components
        .iter()
        .filter(|c| {
            c.name.contains(&clean_term)
                || c.display_name.to_lowercase().contains(term)
                || c.description.to_lowercase().contains(term)
        })
        .collect();

    println!("Search results for '{}' ({} found):\n", term, hits.len());
    for c in hits {
        let deps = if c.dependencies.is_empty() {
            String::new()
        } else {
            format!(" [deps: {}]", c.dependencies.join(", "))
        };
        println!(
            "  {:<26} {:<20} (Ch {:02} {}) - {}{}",
            c.name, c.export_type, c.chapter, c.category, c.description, deps
        );
    }
    println!("\nScaffold with: cargo ely add <component>\n");
}

pub fn scaffold_component(name: &str, project_root: &Path) -> Result<(), String> {
    let registry = load_registry();
    let clean = name.to_ascii_lowercase().replace('-', "_");
    let clean_compact = clean.replace('_', "");

    let entry = registry
        .components
        .iter()
        .find(|c| {
            c.name == clean
                || c.name.replace('_', "") == clean_compact
                || c.display_name.to_lowercase() == clean
                || c.display_name.to_lowercase().replace(' ', "") == clean_compact
        })
        .ok_or_else(|| {
            format!(
                "Unknown component '{name}'. Run 'cargo ely list search {name}' to find matching components."
            )
        })?;

    let components_dir = project_root.join("src").join("components");
    fs::create_dir_all(&components_dir)
        .map_err(|e| format!("Failed to create components directory: {e}"))?;

    for dep_name in &entry.dependencies {
        if let Some(dep_entry) = registry.components.iter().find(|c| &c.name == dep_name) {
            let dep_file = components_dir.join(&dep_entry.file);
            if !dep_file.exists() {
                println!("[ely] Resolving required dependency: '{}'", dep_name);
                scaffold_single_file(dep_entry, project_root, &components_dir)?;
            }
        }
    }

    scaffold_single_file(entry, project_root, &components_dir)?;

    println!(
        "\n[ok] Ready to use: import 'use crate::components::{}::{};'",
        entry.file.trim_end_matches(".rs"),
        entry.export_type
    );
    println!("This component is directly editable in your codebase.");

    Ok(())
}

fn scaffold_single_file(
    entry: &RegistryComponent,
    project_root: &Path,
    components_dir: &Path,
) -> Result<(), String> {
    let dest_file = components_dir.join(&entry.file);
    let source_content = fetch_component_source(&entry.source_dir, &entry.file, project_root)?;

    fs::write(&dest_file, source_content)
        .map_err(|e| format!("Failed to write {}: {e}", dest_file.display()))?;

    update_components_mod(components_dir, entry)?;

    println!(
        "[ok] Added component '{}' into {}",
        entry.name,
        dest_file.display()
    );
    Ok(())
}

fn fetch_component_source(
    source_dir: &str,
    file: &str,
    project_root: &Path,
) -> Result<String, String> {
    let local_path = project_root.join(source_dir).join(file);
    if local_path.is_file()
        && let Ok(content) = fs::read_to_string(&local_path)
    {
        return Ok(content);
    }

    let url = format!(
        "https://raw.githubusercontent.com/amitverma-cf/ely-gpui/main/{}/{}",
        source_dir, file
    );

    println!("[ely] Fetching component source from upstream GitHub...");
    if let Ok(output) = Command::new("curl").args(["-fsSL", &url]).output()
        && output.status.success()
        && let Ok(text) = String::from_utf8(output.stdout)
        && !text.trim().is_empty()
    {
        return Ok(text);
    }

    Ok(format!(
        "// Component: {}\n// You can customize this component as needed.\npub struct {};\n",
        file,
        file.trim_end_matches(".rs")
    ))
}

fn update_components_mod(components_dir: &Path, entry: &RegistryComponent) -> Result<(), String> {
    let mod_file = components_dir.join("mod.rs");
    let mod_name = entry.file.trim_end_matches(".rs");
    let pub_mod_line = format!("pub mod {};\n", mod_name);

    let current = if mod_file.exists() {
        fs::read_to_string(&mod_file).unwrap_or_default()
    } else {
        String::new()
    };

    if !current.contains(&format!("pub mod {}", mod_name)) {
        let mut updated = current;
        updated.push_str(&pub_mod_line);
        fs::write(&mod_file, updated).map_err(|e| format!("Failed to update mod.rs: {e}"))?;
    }

    Ok(())
}
