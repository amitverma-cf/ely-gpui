//! Copies a component and the files it needs into the project as source, with imports rewritten.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::deps::{add_assets, add_crates};
use crate::graph::Tree;
use crate::rewrite::transform;

pub struct Options {
    pub from: Option<PathBuf>,
    pub dir: PathBuf,
    pub force: bool,
}

pub const DEFAULT_SOURCE: &str = "https://github.com/amitverma-cf/mok-gpui";

/// Where the library's source is: a given checkout, the current one, or a fresh shallow clone.
pub fn source_root(from: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(dir) = from {
        return Ok(dir.to_path_buf());
    }
    let here = Path::new(".");
    if here.join("registry.json").is_file() && here.join("src/lib.rs").is_file() {
        return Ok(here.to_path_buf());
    }
    let url = std::env::var("MOK_SOURCE_URL").unwrap_or_else(|_| DEFAULT_SOURCE.into());
    let cache = std::env::temp_dir().join(format!("mok-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&cache);
    println!("[mok] Fetching the library source from {url}");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--quiet", &url])
        .arg(&cache)
        .status()
        .map_err(|e| format!("git is needed to fetch the source: {e}"))?;
    if !status.success() {
        return Err(format!(
            "git clone {url} failed; pass --from <checkout> to use a local copy"
        ));
    }
    Ok(cache)
}

fn fnv(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

struct Lock {
    path: PathBuf,
    files: BTreeMap<String, (String, String, String)>, // dest path -> (unit, hash of the file, hash of what was generated)
}

impl Lock {
    fn read(root: &Path) -> Lock {
        let path = root.join("mok.lock");
        let mut files = BTreeMap::new();
        if let Ok(text) = fs::read_to_string(&path)
            && let Ok(value) = text.parse::<toml::Value>()
            && let Some(list) = value.get("file").and_then(|f| f.as_array())
        {
            for f in list {
                let get = |k: &str| f.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                files.insert(get("path"), (get("unit"), get("hash"), get("src")));
            }
        }
        Lock { path, files }
    }

    fn write(&self) -> Result<(), String> {
        let mut out = String::from(
            "# Written by `cargo mok`. Records what was copied, so edits are kept on update.\n",
        );
        for (path, (unit, hash, src)) in &self.files {
            out.push_str(&format!(
                "\n[[file]]\npath = {path:?}\nunit = {unit:?}\nhash = {hash:?}\nsrc = {src:?}\n"
            ));
        }
        fs::write(&self.path, out).map_err(|e| format!("{}: {e}", self.path.display()))
    }
}

pub struct Registry {
    pub assets_of: BTreeMap<String, Vec<String>>,
    pub units: BTreeMap<String, Vec<String>>,
    pub crates_of: BTreeMap<String, Vec<String>>,
    pub specs: Value,
    pub chapters: BTreeSet<String>,
}

fn load(root: &Path) -> Result<Registry, String> {
    let path = root.join("registry.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let units = value["units"]
        .as_object()
        .ok_or("registry.json has no file graph; run `cargo mok registry` in the library")?;
    let mut out = BTreeMap::new();
    let mut crates_of = BTreeMap::new();
    let mut assets_of = BTreeMap::new();
    let mut chapters = BTreeSet::new();
    for (id, u) in units {
        let req = u["requires"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(ch) = id.split('/').next() {
            chapters.insert(ch.to_string());
        }
        let used = u["crates"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        crates_of.insert(id.clone(), used);
        let files: Vec<String> = u["assets"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if !files.is_empty() {
            assets_of.insert(id.clone(), files);
        }
        out.insert(id.clone(), req);
    }
    Ok(Registry {
        assets_of,
        units: out,
        crates_of,
        specs: value["crates"].clone(),
        chapters,
    })
}

/// The units a unit needs to compile: itself, its requirements, and every module up its path.
fn closure(reg: &Registry, start: &[String]) -> Result<BTreeSet<String>, String> {
    let mut set = BTreeSet::new();
    let mut work: Vec<String> = start.to_vec();
    while let Some(id) = work.pop() {
        if !set.insert(id.clone()) {
            continue;
        }
        let needs = reg
            .units
            .get(&id)
            .ok_or_else(|| format!("registry has no unit {id}"))?;
        work.extend(needs.iter().cloned());
        let parts: Vec<&str> = id.split('/').collect();
        for depth in 1..parts.len() {
            let parent = format!("{}/mod.rs", parts[..depth].join("/"));
            if parent != id && reg.units.contains_key(&parent) {
                work.push(parent);
            }
        }
    }
    Ok(set)
}

pub fn install(root: &Path, units: &[String], opts: &Options) -> Result<(), String> {
    let source = source_root(opts.from.as_deref())?;
    let reg = load(&source)?;
    let tree = Tree::load(&source.join("src"))?;
    let mut lock = Lock::read(root);
    let mut wanted: BTreeSet<String> = lock.files.values().map(|(u, _, _)| u.clone()).collect();
    let before = wanted.len();
    wanted.extend(closure(&reg, units)?);
    let all = closure(&reg, &wanted.iter().cloned().collect::<Vec<_>>())?;
    let dest_root = root.join(&opts.dir);
    let rel_dir = opts.dir.to_string_lossy().replace('\\', "/");
    let (mut wrote, mut kept, mut same) = (0, 0, 0);
    for unit in &all {
        let text = fs::read_to_string(source.join("src").join(unit))
            .map_err(|e| format!("{unit}: {e}"))?;
        let mp = tree
            .mods
            .iter()
            .find(|(_, m)| m.unit == *unit)
            .map(|(k, _)| k.clone())
            .ok_or_else(|| format!("{unit}: not in the source tree"))?;
        let out = transform(&text, unit, &all, &reg.chapters, &tree, &mp)?;
        let dest = dest_root.join(unit);
        let key = format!("{rel_dir}/{unit}");
        let managed = unit.ends_with("mod.rs");
        let generated = fnv(&out);
        if let Ok(existing) = fs::read_to_string(&dest) {
            let entry = lock.files.get(&key);
            let untouched = entry.is_some_and(|(_, hash, _)| *hash == fnv(&existing));
            if untouched && entry.is_some_and(|(_, _, src)| *src == generated) {
                same += 1;
                continue;
            }
            if !untouched && !managed && !opts.force {
                println!("[keep] {key}: edited here, left as it is (--force replaces it)");
                kept += 1;
                continue;
            }
        }
        fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&dest, &out).map_err(|e| format!("{}: {e}", dest.display()))?;
        lock.files
            .insert(key.clone(), (unit.clone(), String::new(), generated));
        wrote += 1;
    }
    write_index(&dest_root, &all, &reg.chapters)?;
    rustfmt(&dest_root.join("mod.rs"));
    for (key, entry) in lock.files.iter_mut() {
        if let Ok(text) = fs::read_to_string(root.join(key)) {
            entry.1 = fnv(&text);
        }
    }
    lock.write()?;
    add_assets(root, &source, &reg, &all)?;
    add_crates(root, &reg, &all, opts)?;
    println!(
        "[ok] {wrote} files written, {same} already current, {kept} kept (edited); {} new units",
        all.len().saturating_sub(before)
    );
    println!("\nIn your crate: `mod ui;` then, once after mok_gpui::init, `ui::init(cx);`");
    Ok(())
}

fn rustfmt(path: &Path) {
    let _ = Command::new("rustfmt")
        .args(["--edition", "2024", "--quiet"])
        .arg(path)
        .output();
}

/// `ui/mod.rs`: the chapters present and one call that binds their keys.
fn write_index(
    dest: &Path,
    units: &BTreeSet<String>,
    chapters: &BTreeSet<String>,
) -> Result<(), String> {
    let present: BTreeSet<&str> = units
        .iter()
        .filter_map(|u| u.split('/').next())
        .filter(|c| chapters.contains(*c))
        .collect();
    let mut out = String::from(
        "//! Managed by `cargo mok`; the components live in the folders beside this file.\n#![allow(dead_code, unused_imports)]\n\n",
    );
    let mut binds = vec![];
    for chapter in &present {
        out.push_str(&format!("pub mod {chapter};\n"));
        let index = dest.join(chapter).join("mod.rs");
        if fs::read_to_string(&index).is_ok_and(|t| t.contains("fn bind_keys")) {
            binds.push(*chapter);
        }
    }
    out.push_str(
        "\n/// Binds the keys the copied components use.\npub fn init(cx: &mut gpui::App) {\n",
    );
    let binds_none = binds.is_empty();
    for chapter in binds {
        out.push_str(&format!("    {chapter}::bind_keys(cx);\n"));
    }
    if binds_none {
        out.push_str("    let _ = cx;\n");
    }
    out.push_str("}\n");
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    fs::write(dest.join("mod.rs"), out).map_err(|e| e.to_string())?;
    Ok(())
}
