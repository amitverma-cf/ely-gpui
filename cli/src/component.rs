use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    #[serde(default)]
    pub unit: Option<String>,
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
    const EMBEDDED: &str = include_str!("../../registry.json");
    serde_json::from_str(EMBEDDED).expect("Valid embedded registry.json")
}

pub fn list_components(filter: Option<&str>, as_json: bool) {
    if as_json {
        if let Ok(content) = fs::read_to_string("registry.json") {
            println!("{content}");
        } else {
            const EMBEDDED: &str = include_str!("../../registry.json");
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

pub fn scaffold_components(
    names: &[String],
    project_root: &Path,
    opts: &crate::install::Options,
) -> Result<(), String> {
    let source = crate::install::source_root(opts.from.as_deref())?;
    let registry = load_registry_from(&source);
    let mut units = vec![];
    for name in names {
        let clean = name.to_ascii_lowercase().replace('-', "_");
        let compact = clean.replace('_', "");
        let entry = registry
            .components
            .iter()
            .find(|c| {
                c.name == clean
                    || c.name.replace('_', "") == compact
                    || c.display_name.to_lowercase() == clean
                    || c.display_name.to_lowercase().replace(' ', "") == compact
            })
            .ok_or_else(|| {
                format!("Unknown component '{name}'. Run 'cargo ely list search {name}' to find matching components.")
            })?;
        let unit = entry.unit.clone().ok_or_else(|| {
            format!(
                "'{}' ships in the ely-gpui crate itself (theme, motion, primitives, i18n, layout, typography): use ely_gpui::{}",
                entry.name,
                entry.source_dir.trim_start_matches("src/")
            )
        })?;
        units.push(unit);
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

fn load_registry_from(source: &Path) -> Registry {
    if let Ok(content) = fs::read_to_string(source.join("registry.json"))
        && let Ok(reg) = serde_json::from_str::<Registry>(&content)
    {
        return reg;
    }
    load_registry()
}
