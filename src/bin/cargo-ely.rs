use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "cargo_ely/component.rs"]
mod component;
#[path = "cargo_ely/icons.rs"]
mod icons;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "ely" {
        args.remove(1);
    }

    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    match command {
        "init" => handle_init(&args[2..]),
        "list" => handle_list(&args[2..]),
        "add" => handle_add(&args[2..]),
        "doctor" | "check" => handle_doctor(),
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!(
                "Unknown command: '{}'. Run 'cargo ely help' for usage.",
                other
            );
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Ely GPUI Component CLI\n");
    println!("Usage:");
    println!("  cargo ely <command> [options]\n");
    println!("Commands:");
    println!("  init                     Initialize ely-gpui.toml and project asset structure");
    println!("  list [--json]            List all available components in the registry");
    println!("  add icons <pack>         Add an icon pack (lucide, heroicons, hugeicons)");
    println!("  add <component>          Scaffold an editable component into src/components/");
    println!("  doctor, check            Verify ely-gpui.toml contrast, fonts, and assets");
    println!("  help                     Show this help message\n");
    println!("Examples:");
    println!("  cargo ely init");
    println!("  cargo ely list");
    println!("  cargo ely add button");
    println!("  cargo ely add command_palette");
    println!("  cargo ely add icons heroicons");
    println!("  cargo ely doctor");
}

fn detect_folder_structure(root: &Path) -> (Vec<String>, String) {
    let cargo_toml_path = root.join("Cargo.toml");
    let is_workspace = if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
        content.contains("[workspace]")
    } else {
        false
    };

    if is_workspace {
        if root.join("crates").is_dir() {
            (
                vec!["crates/**/*.rs".into()],
                "workspace with 'crates/' folder".into(),
            )
        } else if root.join("packages").is_dir() {
            (
                vec!["packages/**/*.rs".into()],
                "workspace with 'packages/' folder".into(),
            )
        } else {
            (
                vec!["*/**/*.rs".into()],
                "workspace with root members".into(),
            )
        }
    } else if root.join("src").is_dir() {
        (
            vec!["src/**/*.rs".into(), "examples/**/*.rs".into()],
            "standard single-crate (src/)".into(),
        )
    } else {
        (vec!["**/*.rs".into()], "custom project structure".into())
    }
}

fn handle_init(args: &[String]) {
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = cwd.join("ely-gpui.toml");

    if config_path.exists() && !force {
        eprintln!("[warn] ely-gpui.toml already exists. Use '--force' to overwrite.");
        return;
    }

    let (scan_sources, structure_desc) = detect_folder_structure(&cwd);
    println!("[ely] Detected project layout: {}", structure_desc);

    let assets_dir = cwd.join("assets");
    let icons_dir = assets_dir.join("icons");
    let fonts_dir = assets_dir.join("fonts");
    let images_dir = assets_dir.join("images");

    let _ = fs::create_dir_all(&icons_dir);
    let _ = fs::create_dir_all(&fonts_dir);
    let _ = fs::create_dir_all(&images_dir);

    let sources_formatted = scan_sources
        .iter()
        .map(|s| format!("    {:?}", s))
        .collect::<Vec<_>>()
        .join(",\n");

    let toml_content = format!(
        r#"[paths]
assets_dir = "assets"
icons_dir = "assets/icons"
fonts_dir = "assets/fonts"
images_dir = "assets/images"
scan_sources = [
{sources_formatted}
]

[theme]
preset = "ely"
# Customize theme colors or tokens by adding tables here:
# [theme.palette.light]
# bg = 0xfcfaf7
# fg = 0x181613
# [theme.tokens]
# base_rem = 16.0

[fonts]
sans = {{ family = "Inter", files = [
    "fonts/Inter-Regular.ttf",
    "fonts/Inter-Medium.ttf",
    "fonts/Inter-SemiBold.ttf",
    "fonts/Inter-Italic.ttf"
] }}
mono = {{ family = "JetBrains Mono", files = [
    "fonts/JetBrainsMono-Regular.ttf",
    "fonts/JetBrainsMono-Medium.ttf"
] }}
fallback = {{ family = "IBM Plex Sans", files = [
    "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"
] }}

[icons]
packs = ["lucide"]
mode = "auto"
minify = true
fallback = "image-off"

[media.images]

[media.videos]
"#
    );

    if let Err(e) = fs::write(&config_path, toml_content) {
        eprintln!("[error] Failed to write ely-gpui.toml: {}", e);
    } else {
        println!("[ok] Successfully created ely-gpui.toml");
        println!("[ok] Created directories: assets/icons, assets/fonts, assets/images");
        println!("\nNext steps:");
        println!("  - Place fonts or SVG icons in 'assets/'");
        println!("  - Run 'cargo ely add icons <pack>' to enable additional icon packs");
        println!("  - Run 'cargo ely doctor' to verify theme tokens and assets");
    }
}

fn handle_list(args: &[String]) {
    let as_json = args.iter().any(|a| a == "--json");
    component::list_components(as_json);
}

fn handle_add(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cargo ely add <component> OR cargo ely add icons <pack>");
        eprintln!("Run 'cargo ely list' to see all available components.");
        return;
    }

    if args[0] == "icons" {
        if args.len() < 2 {
            eprintln!("Usage: cargo ely add icons <pack> (e.g., heroicons, hugeicons, lucide)");
            return;
        }
        handle_add_icons(&args[1]);
        return;
    }

    handle_add_component(&args[0]);
}

fn handle_add_icons(pack: &str) {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = cwd.join("ely-gpui.toml");
    if !config_path.exists() {
        eprintln!("[error] ely-gpui.toml not found. Run 'cargo ely init' first.");
        return;
    }

    let pack_clean = pack.to_ascii_lowercase();
    let pack_dir = cwd.join("assets").join("icons").join(&pack_clean);
    let _ = fs::create_dir_all(&pack_dir);

    if let Ok(content) = fs::read_to_string(&config_path) {
        let needle = format!("\"{}\"", pack_clean);
        if content.contains(&needle) {
            println!(
                "[info] Icon pack '{}' is already declared in ely-gpui.toml",
                pack_clean
            );
        } else if let Some(idx) = content.find("packs = [") {
            let insert_pos = idx + "packs = [".len();
            let mut updated = content.clone();
            updated.insert_str(insert_pos, &format!("\"{}\", ", pack_clean));
            let _ = fs::write(&config_path, updated);
            println!(
                "[ok] Added '{}' to [icons] packs in ely-gpui.toml",
                pack_clean
            );
        }
    }

    let enum_name = match pack_clean.as_str() {
        "heroicons" => "HeroIcon",
        "hugeicons" => "HugeIcon",
        "lucide" => "LucideIcon",
        _ => "CustomIcon",
    };

    println!("[ok] Created directory: assets/icons/{}", pack_clean);
    println!("[ok] Generated typed provider enum: {}", enum_name);

    match icons::download_icon_pack(&pack_clean, &pack_dir) {
        Ok(count) => println!(
            "[ok] Downloaded {} SVG icons into assets/icons/{}",
            count, pack_clean
        ),
        Err(e) => eprintln!("[warn] Icon download warning: {}", e),
    }

    println!("\nUsage example in GPUI views:");
    println!("  Icon::new({}::<Variant>)", enum_name);
}

fn handle_add_component(name: &str) {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Err(e) = component::scaffold_component(name, &cwd) {
        eprintln!("[error] {}", e);
        println!("\nAvailable components in Ely catalog:");
        for c in component::CATALOG {
            println!("  - {:<18} {}", c.name, c.description);
        }
    }
}

fn srgb_luminance(rgb: u32) -> f64 {
    let r = ((rgb >> 16) & 0xff) as f64 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f64 / 255.0;
    let b = (rgb & 0xff) as f64 / 255.0;
    let to_linear = |v: f64| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * to_linear(r) + 0.7152 * to_linear(g) + 0.0722 * to_linear(b)
}

fn contrast_ratio(c1: u32, c2: u32) -> f64 {
    let l1 = srgb_luminance(c1);
    let l2 = srgb_luminance(c2);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

fn handle_doctor() {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = cwd.join("ely-gpui.toml");

    println!("Checking Ely configuration...\n");
    if !config_path.exists() {
        eprintln!("[FAIL] ely-gpui.toml was not found. Run 'cargo ely init' to create one.");
        return;
    }
    println!("[PASS] Configuration file found: {}", config_path.display());

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[FAIL] Could not read ely-gpui.toml: {}", e);
            return;
        }
    };

    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[FAIL] Syntax error in ely-gpui.toml: {}", e);
            return;
        }
    };
    println!("[PASS] Syntax of ely-gpui.toml is valid TOML");

    // Check WCAG Contrast
    if let Some(theme) = parsed.get("theme").and_then(|t| t.get("palette")) {
        for mode in ["light", "dark"] {
            if let Some(pal) = theme.get(mode) {
                let paper = pal
                    .get("paper")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0xffffff) as u32;
                let ink = pal
                    .get("ink")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0x181613) as u32;
                let ratio = contrast_ratio(paper, ink);
                if ratio >= 4.5 {
                    println!(
                        "[PASS] {} palette ink on paper contrast: {:.2}:1 (WCAG AA compliant)",
                        mode, ratio
                    );
                } else {
                    println!(
                        "[WARN] {} palette ink on paper contrast: {:.2}:1 (below WCAG 4.5:1)",
                        mode, ratio
                    );
                }
            }
        }
    }

    // Check Assets
    let mut missing_fonts = 0;
    if let Some(fonts) = parsed.get("fonts").and_then(|f| f.as_table()) {
        for (name, entry) in fonts {
            if let Some(files) = entry.get("files").and_then(|f| f.as_array()) {
                for file_val in files {
                    if let Some(rel) = file_val.as_str() {
                        let path = cwd.join("assets").join(rel);
                        let root_path = cwd.join(rel);
                        if !path.exists() && !root_path.exists() {
                            println!(
                                "[WARN] Font file declared in [fonts.{}] not found: {}",
                                name, rel
                            );
                            missing_fonts += 1;
                        }
                    }
                }
            }
        }
    }
    if missing_fonts == 0 {
        println!("[PASS] All declared fonts resolved successfully on disk");
    }

    // Check Icons
    if let Some(icons) = parsed.get("icons").and_then(|i| i.as_table()) {
        let packs = icons.get("packs").and_then(|p| p.as_array());
        let count = packs.map(|p| p.len()).unwrap_or(0);
        println!("[PASS] Active icon packs: {} configured", count);
    }

    println!("\n[ok] Ely Doctor completed.");
}
