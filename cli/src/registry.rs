//! `cargo ely registry`: rewrites `registry.json`'s file graph from the library's source.

use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::graph::{self, CORE, Tree};

pub fn generate(repo: &Path) -> Result<(), String> {
    let src = repo.join("src");
    let tree = Tree::load(&src)?;
    let deps = dependencies(repo)?;
    let mut units = Map::new();
    let mut extra: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (mp, m) in &tree.mods {
        let Some(chapter) = mp.first() else { continue };
        if !tree.is_chapter(chapter) {
            continue;
        }
        let needs = graph::needs(&tree, mp);
        for target in &needs.extends {
            extra
                .entry(target.clone())
                .or_default()
                .push(m.unit.clone());
        }
        units.insert(
            m.unit.clone(),
            json!({
                "requires": needs.requires,
                "core": needs.core,
                "crates": needs.crates.iter().filter(|c| deps.contains_key(*c)).collect::<Vec<_>>(),
            }),
        );
    }
    for (target, adds) in extra {
        if let Some(unit) = units.get_mut(&target) {
            let list = unit["requires"].as_array_mut().unwrap();
            for a in adds {
                if !list.iter().any(|v| v == &Value::String(a.clone())) {
                    list.push(Value::String(a));
                }
            }
        }
    }
    let path = repo.join("registry.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut registry: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut missing = vec![];
    if let Some(list) = registry["components"].as_array_mut() {
        for c in list {
            let dir = c["source_dir"]
                .as_str()
                .unwrap_or("")
                .trim_start_matches("src/");
            let unit = format!("{dir}/{}", c["file"].as_str().unwrap_or(""));
            if units.contains_key(&unit) {
                c["unit"] = Value::String(unit);
            } else if CORE.contains(&dir.split('/').next().unwrap_or("")) {
                c["core"] = Value::Bool(true);
            } else {
                missing.push(unit);
            }
        }
    }
    let count = units.len();
    let obj = registry
        .as_object_mut()
        .ok_or("registry.json is not an object")?;
    obj.insert("core".into(), json!(CORE));
    obj.insert("crates".into(), Value::Object(deps));
    obj.insert("units".into(), Value::Object(units));
    fs::write(
        &path,
        serde_json::to_string_pretty(&registry).unwrap() + "\n",
    )
    .map_err(|e| e.to_string())?;
    println!("[ok] wrote {count} units to {}", path.display());
    for unit in &missing {
        eprintln!("[warn] registry names a file the source lacks: {unit}");
    }
    Ok(())
}

/// The library's dependencies as `cargo add` needs them, by the name code uses.
fn dependencies(repo: &Path) -> Result<Map<String, Value>, String> {
    let text = fs::read_to_string(repo.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let manifest: toml::Value = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
    let mut out = Map::new();
    let mut take = |table: &toml::Value, target: Option<&str>| {
        let Some(table) = table.as_table() else {
            return;
        };
        for (key, spec) in table {
            let (version, package, features, default) = match spec {
                toml::Value::String(v) => (v.clone(), None, vec![], true),
                toml::Value::Table(t) => (
                    t.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string(),
                    t.get("package").and_then(|v| v.as_str()).map(String::from),
                    t.get("features")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|f| f.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    t.get("default-features")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                ),
                _ => continue,
            };
            let name = key.replace('-', "_");
            out.entry(name).or_insert(json!({
                "key": key, "version": version, "package": package,
                "features": features, "default_features": default, "target": target,
            }));
        }
    };
    if let Some(t) = manifest.get("dependencies") {
        take(t, None);
    }
    if let Some(targets) = manifest.get("target").and_then(|t| t.as_table()) {
        for (cfg, body) in targets {
            if let Some(t) = body.get("dependencies") {
                take(t, Some(cfg));
            }
        }
    }
    Ok(out)
}
