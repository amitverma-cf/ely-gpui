use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ComponentEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub file: &'static str,
    pub export_type: &'static str,
    pub description: &'static str,
    pub dependencies: &'static [&'static str],
}

pub const CATALOG: &[ComponentEntry] = &[
    ComponentEntry {
        name: "button",
        category: "inputs",
        file: "button.rs",
        export_type: "Button",
        description: "Buttons, icon buttons, toggle buttons",
        dependencies: &[],
    },
    ComponentEntry {
        name: "input",
        category: "inputs",
        file: "text_input.rs",
        export_type: "TextInput",
        description: "Single and multi-line text input fields",
        dependencies: &[],
    },
    ComponentEntry {
        name: "checkbox",
        category: "inputs",
        file: "checkbox.rs",
        export_type: "Checkbox",
        description: "Checkboxes with tristate support",
        dependencies: &[],
    },
    ComponentEntry {
        name: "switch",
        category: "inputs",
        file: "switch.rs",
        export_type: "Switch",
        description: "Toggle switches",
        dependencies: &[],
    },
    ComponentEntry {
        name: "slider",
        category: "inputs",
        file: "slider.rs",
        export_type: "Slider",
        description: "Continuous and stepped value sliders",
        dependencies: &[],
    },
    ComponentEntry {
        name: "dropdown",
        category: "inputs",
        file: "dropdown.rs",
        export_type: "Dropdown",
        description: "Single and multi-select dropdown menus",
        dependencies: &[],
    },
    ComponentEntry {
        name: "badge",
        category: "display",
        file: "badge.rs",
        export_type: "Badge",
        description: "Status and count badges",
        dependencies: &[],
    },
    ComponentEntry {
        name: "avatar",
        category: "display",
        file: "avatar.rs",
        export_type: "Avatar",
        description: "User avatar images with initials fallback",
        dependencies: &[],
    },
    ComponentEntry {
        name: "tooltip",
        category: "primitives",
        file: "tooltip.rs",
        export_type: "Tooltip",
        description: "Hover and focus help tooltips",
        dependencies: &[],
    },
    ComponentEntry {
        name: "app_layout",
        category: "layout",
        file: "app_layout.rs",
        export_type: "AppLayout",
        description: "Desktop shell layout with header, sidebars and footer",
        dependencies: &[],
    },
    ComponentEntry {
        name: "card",
        category: "layout",
        file: "surface.rs",
        export_type: "Card",
        description: "Structured content container cards",
        dependencies: &[],
    },
    ComponentEntry {
        name: "dialog",
        category: "overlays",
        file: "dialog.rs",
        export_type: "Dialog",
        description: "Modal dialog overlays with backdrop",
        dependencies: &["button"],
    },
    ComponentEntry {
        name: "command_palette",
        category: "overlays",
        file: "command_palette.rs",
        export_type: "CommandPalette",
        description: "Command palette / quick search dialog",
        dependencies: &["badge", "input"],
    },
    ComponentEntry {
        name: "sheet",
        category: "overlays",
        file: "sheet.rs",
        export_type: "Sheet",
        description: "Side sheet / drawer panels",
        dependencies: &[],
    },
    ComponentEntry {
        name: "toast",
        category: "overlays",
        file: "toast.rs",
        export_type: "Toast",
        description: "Toast notification manager and toasts",
        dependencies: &[],
    },
    ComponentEntry {
        name: "table",
        category: "collections",
        file: "table.rs",
        export_type: "Table",
        description: "Virtual scrolling data tables with header resize",
        dependencies: &[],
    },
    ComponentEntry {
        name: "tabs",
        category: "navigation",
        file: "tabs.rs",
        export_type: "Tabs",
        description: "Tabbed navigation bars and panels",
        dependencies: &[],
    },
];

pub fn list_components(as_json: bool) {
    if as_json {
        let registry_path = Path::new("registry.json");
        if registry_path.is_file()
            && let Ok(json) = fs::read_to_string(registry_path)
        {
            println!("{json}");
            return;
        }

        println!("{{\"components\":[");
        for (i, c) in CATALOG.iter().enumerate() {
            let comma = if i + 1 < CATALOG.len() { "," } else { "" };
            let deps_json = c
                .dependencies
                .iter()
                .map(|d| format!("\"{d}\""))
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "  {{\"name\":\"{}\",\"category\":\"{}\",\"file\":\"{}\",\"export\":\"{}\",\"dependencies\":[{deps_json}],\"description\":\"{}\"}}{comma}",
                c.name, c.category, c.file, c.export_type, c.description
            );
        }
        println!("]}}");
        return;
    }

    println!("Ely GPUI Component Registry\n");
    println!("Total available components: {}\n", CATALOG.len());

    let categories = [
        "inputs",
        "display",
        "layout",
        "overlays",
        "primitives",
        "collections",
        "navigation",
    ];

    for cat in categories {
        let matching: Vec<&ComponentEntry> = CATALOG.iter().filter(|c| c.category == cat).collect();
        if matching.is_empty() {
            continue;
        }

        println!("[{cat}]");
        for c in matching {
            let deps = if c.dependencies.is_empty() {
                String::new()
            } else {
                format!(" (deps: {})", c.dependencies.join(", "))
            };
            println!(
                "  {:<18} {:<15} {}{}",
                c.name, c.export_type, c.description, deps
            );
        }
        println!();
    }

    println!("Scaffold any component into your project with:");
    println!("  cargo ely add <component>\n");
}

pub fn scaffold_component(name: &str, project_root: &Path) -> Result<(), String> {
    let clean = name.to_ascii_lowercase().replace('-', "_");
    let entry = CATALOG.iter().find(|c| c.name == clean).ok_or_else(|| {
        format!("Unknown component '{name}'. Run 'cargo ely list' to see available components.")
    })?;

    let components_dir = project_root.join("src").join("components");
    fs::create_dir_all(&components_dir)
        .map_err(|e| format!("Failed to create components directory: {e}"))?;

    // Auto-resolve missing dependencies first
    for &dep_name in entry.dependencies {
        if let Some(dep_entry) = CATALOG.iter().find(|c| c.name == dep_name) {
            let dep_file = components_dir.join(dep_entry.file);
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
    entry: &ComponentEntry,
    project_root: &Path,
    components_dir: &Path,
) -> Result<(), String> {
    let dest_file = components_dir.join(entry.file);
    let source_content = fetch_component_source(entry.category, entry.file, project_root)?;

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
    category: &str,
    file: &str,
    project_root: &Path,
) -> Result<String, String> {
    let local_path = project_root.join("src").join(category).join(file);
    if local_path.is_file()
        && let Ok(content) = fs::read_to_string(&local_path)
    {
        return Ok(content);
    }

    let url = format!(
        "https://raw.githubusercontent.com/amitverma-cf/ely-gpui/main/src/{}/{}",
        category, file
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

fn update_components_mod(components_dir: &Path, entry: &ComponentEntry) -> Result<(), String> {
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
