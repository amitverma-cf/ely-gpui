//! Copies a component and the files it needs into the project as source, with imports rewritten.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Item, UseTree};

use crate::graph::{CORE, Mp, Tree, is_test};

pub struct Options {
    pub from: Option<PathBuf>,
    pub dir: PathBuf,
    pub force: bool,
}

const DEFAULT_SOURCE: &str = "https://github.com/amitverma-cf/ely-gpui";

/// Where the library's source is: a given checkout, the current one, or a fresh shallow clone.
pub fn source_root(from: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(dir) = from {
        return Ok(dir.to_path_buf());
    }
    let here = Path::new(".");
    if here.join("registry.json").is_file() && here.join("src/lib.rs").is_file() {
        return Ok(here.to_path_buf());
    }
    let url = std::env::var("ELY_SOURCE_URL").unwrap_or_else(|_| DEFAULT_SOURCE.into());
    let cache = std::env::temp_dir().join(format!("ely-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&cache);
    println!("[ely] Fetching the library source from {url}");
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
        let path = root.join("ely.lock");
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
            "# Written by `cargo ely`. Records what was copied, so edits are kept on update.\n",
        );
        for (path, (unit, hash, src)) in &self.files {
            out.push_str(&format!(
                "\n[[file]]\npath = {path:?}\nunit = {unit:?}\nhash = {hash:?}\nsrc = {src:?}\n"
            ));
        }
        fs::write(&self.path, out).map_err(|e| format!("{}: {e}", self.path.display()))
    }
}

struct Registry {
    units: BTreeMap<String, Vec<String>>,
    crates_of: BTreeMap<String, Vec<String>>,
    specs: Value,
    chapters: BTreeSet<String>,
}

fn load(root: &Path) -> Result<Registry, String> {
    let path = root.join("registry.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let units = value["units"]
        .as_object()
        .ok_or("registry.json has no file graph; run `cargo ely registry` in the library")?;
    let mut out = BTreeMap::new();
    let mut crates_of = BTreeMap::new();
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
        out.insert(id.clone(), req);
    }
    Ok(Registry {
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
    add_crates(root, &reg, &all, opts)?;
    println!(
        "[ok] {wrote} files written, {same} already current, {kept} kept (edited); {} new units",
        all.len().saturating_sub(before)
    );
    println!("\nIn your crate: `mod ui;` then, once after ely_gpui::init, `ui::init(cx);`");
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
        "//! Managed by `cargo ely`; the components live in the folders beside this file.\n\n",
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

struct Edit {
    start: usize,
    end: usize,
    text: String,
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    v.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    v
}

fn offset(text: &str, starts: &[usize], at: proc_macro2::LineColumn) -> usize {
    let line = starts[at.line - 1];
    let rest = &text[line..];
    line + rest
        .char_indices()
        .nth(at.column)
        .map_or(rest.len(), |(i, _)| i)
}

fn range(text: &str, starts: &[usize], span: proc_macro2::Span) -> (usize, usize) {
    (
        offset(text, starts, span.start()),
        offset(text, starts, span.end()),
    )
}

fn print(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(p) => format!("{}::{}", p.ident, print(&p.tree)),
        UseTree::Name(n) => n.ident.to_string(),
        UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        UseTree::Glob(_) => "*".into(),
        UseTree::Group(g) => format!(
            "{{{}}}",
            g.items.iter().map(print).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn head(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(p) => Some(p.ident.to_string()),
        UseTree::Name(n) => Some(n.ident.to_string()),
        UseTree::Rename(r) => Some(r.ident.to_string()),
        _ => None,
    }
}

fn prefix_for(name: &str, chapters: &BTreeSet<String>) -> &'static str {
    if chapters.contains(name) {
        "crate::ui"
    } else {
        "ely_gpui"
    }
}

/// `use crate::...` split between the copied chapters and the crate that stays.
fn rewrite_use(tree: &UseTree, chapters: &BTreeSet<String>) -> Option<Vec<String>> {
    let UseTree::Path(root) = tree else {
        return None;
    };
    if root.ident != "crate" {
        return None;
    }
    let entries: Vec<&UseTree> = match &*root.tree {
        UseTree::Group(g) => g.items.iter().collect(),
        other => vec![other],
    };
    let (mut ui, mut core) = (vec![], vec![]);
    for e in entries {
        let name = head(e).unwrap_or_default();
        if prefix_for(&name, chapters) == "crate::ui" {
            ui.push(print(e));
        } else {
            core.push(print(e));
        }
    }
    let mut out = vec![];
    for (prefix, list) in [("ely_gpui", core), ("crate::ui", ui)] {
        match list.len() {
            0 => {}
            1 => out.push(format!("{prefix}::{}", list[0])),
            _ => out.push(format!("{prefix}::{{{}}}", list.join(", "))),
        }
    }
    Some(out)
}

/// The use tree without the leaves `keep` refuses; `None` when none is left.
fn prune(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    keep: &dyn Fn(&[String]) -> bool,
) -> Option<UseTree> {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            let inner = prune(&p.tree, prefix, keep);
            prefix.pop();
            inner.map(|tree| {
                let mut out = p.clone();
                out.tree = Box::new(tree);
                UseTree::Path(out)
            })
        }
        UseTree::Name(n) => {
            let mut path = prefix.clone();
            if n.ident != "self" {
                path.push(n.ident.to_string());
            }
            keep(&path).then(|| tree.clone())
        }
        UseTree::Rename(r) => {
            let mut path = prefix.clone();
            if r.ident != "self" {
                path.push(r.ident.to_string());
            }
            keep(&path).then(|| tree.clone())
        }
        UseTree::Glob(_) => keep(prefix).then(|| tree.clone()),
        UseTree::Group(g) => {
            let items: Vec<UseTree> = g
                .items
                .iter()
                .filter_map(|i| prune(i, prefix, keep))
                .collect();
            match items.len() {
                0 => None,
                _ => {
                    let mut out = g.clone();
                    out.items = items.into_iter().collect();
                    Some(UseTree::Group(out))
                }
            }
        }
    }
}

struct Uses<'a>(Vec<&'a syn::ItemUse>);

impl<'ast> Visit<'ast> for Uses<'ast> {
    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        self.0.push(u);
    }
}

/// One file as the project gets it.
pub fn transform(
    text: &str,
    unit: &str,
    installed: &BTreeSet<String>,
    chapters: &BTreeSet<String>,
    tree: &Tree,
    mp: &Mp,
) -> Result<String, String> {
    let ast = syn::parse_file(text).map_err(|e| format!("{unit}: {e}"))?;
    let starts = line_starts(text);
    let mut edits: Vec<Edit> = vec![];
    let dir = if unit.ends_with("/mod.rs") {
        unit.trim_end_matches("/mod.rs").to_string()
    } else {
        unit.trim_end_matches(".rs").to_string()
    };
    let present = |name: &str| {
        installed.contains(&format!("{dir}/{name}.rs"))
            || installed.contains(&format!("{dir}/{name}/mod.rs"))
    };
    for item in &ast.items {
        let attrs = match item {
            Item::Mod(i) => &i.attrs,
            Item::Use(i) => &i.attrs,
            Item::Fn(i) => &i.attrs,
            Item::Impl(i) => &i.attrs,
            Item::Const(i) => &i.attrs,
            Item::Struct(i) => &i.attrs,
            Item::Enum(i) => &i.attrs,
            _ => continue,
        };
        let (start, end) = range(text, &starts, item.span());
        let drop = |edits: &mut Vec<Edit>| {
            edits.push(Edit {
                start,
                end,
                text: String::new(),
            });
        };
        if is_test(attrs) {
            drop(&mut edits);
            continue;
        }
        match item {
            Item::Mod(m) if m.content.is_none() && !present(&m.ident.to_string()) => {
                drop(&mut edits)
            }
            _ => {}
        }
    }
    let mut uses = Uses(vec![]);
    uses.visit_file(&ast);
    for u in uses.0 {
        let (start, end) = range(text, &starts, u.span());
        if edits.iter().any(|e| e.start <= start && end <= e.end) {
            continue;
        }
        let pruned = prune(&u.tree, &mut vec![], &|path: &[String]| {
            let Some(abs) = tree.absolute(mp, path) else {
                return true;
            };
            match tree.resolve(&abs) {
                Some(provider) if !provider.is_empty() && !CORE.contains(&provider[0].as_str()) => {
                    installed.contains(&tree.mods[&provider].unit)
                }
                _ => true,
            }
        });
        let Some(kept) = pruned else {
            edits.push(Edit {
                start,
                end,
                text: String::new(),
            });
            continue;
        };
        let changed = print(&kept) != print(&u.tree);
        let parts = rewrite_use(&kept, chapters);
        if !changed && parts.is_none() {
            continue;
        }
        let parts = parts.unwrap_or_else(|| vec![print(&kept)]);
        let lead = &text[start..range(text, &starts, u.use_token.span).0];
        let line = starts[u.span().start().line - 1];
        let indent: String = text[line..start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let stmts: Vec<String> = parts.iter().map(|p| format!("{lead}use {p};")).collect();
        edits.push(Edit {
            start,
            end,
            text: stmts.join(&format!(
                "
{indent}"
            )),
        });
    }
    edits.sort_by_key(|e| e.start);
    let mut out = String::new();
    let mut at = 0;
    for e in &edits {
        if e.start < at {
            continue;
        }
        out.push_str(&paths(&text[at..e.start], chapters));
        out.push_str(&e.text);
        at = e.end;
    }
    out.push_str(&paths(&text[at..], chapters));
    Ok(out)
}

/// `crate::name` outside a `use`, pointed at where `name` now lives.
fn paths(text: &str, chapters: &BTreeSet<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find("crate::") {
        let before = rest[..i].chars().next_back();
        let boundary = !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '$');
        out.push_str(&rest[..i]);
        let after = &rest[i + 7..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if boundary && !name.is_empty() {
            out.push_str(prefix_for(&name, chapters));
            out.push_str("::");
        } else {
            out.push_str("crate::");
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Keys of the dependencies a project's manifest already names, as code spells them.
fn present(root: &Path) -> BTreeSet<String> {
    let mut have = BTreeSet::new();
    let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return have;
    };
    let Ok(manifest) = text.parse::<toml::Value>() else {
        return have;
    };
    let mut tables = vec![manifest.get("dependencies").cloned()];
    if let Some(t) = manifest.get("target").and_then(|t| t.as_table()) {
        tables.extend(t.values().map(|b| b.get("dependencies").cloned()));
    }
    for table in tables.into_iter().flatten() {
        if let Some(t) = table.as_table() {
            for (key, spec) in t {
                have.insert(key.replace('-', "_"));
                if let Some(rename) = spec.get("package").and_then(|p| p.as_str()) {
                    let _ = rename;
                }
            }
        }
    }
    have
}

/// `cargo add` for every crate the copied files use that the project lacks.
fn add_crates(
    root: &Path,
    reg: &Registry,
    units: &BTreeSet<String>,
    opts: &Options,
) -> Result<(), String> {
    let mut need: BTreeSet<String> = ["gpui".to_string()].into();
    for unit in units {
        need.extend(reg.crates_of.get(unit).into_iter().flatten().cloned());
    }
    let have = present(root);
    if !have.contains("ely_gpui") {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(root).args(["add", "ely-gpui"]);
        match &opts.from {
            Some(path) if path.join("Cargo.toml").is_file() => {
                cmd.arg("--path")
                    .arg(fs::canonicalize(path).map_err(|e| e.to_string())?);
            }
            _ => {
                cmd.args(["--git", DEFAULT_SOURCE]);
            }
        }
        run(cmd, "ely-gpui")?;
    }
    for name in need.iter().filter(|n| !have.contains(*n)) {
        let Some(spec) = reg.specs.get(name) else {
            continue;
        };
        let key = spec["key"].as_str().unwrap_or(name);
        let package = spec["package"].as_str().unwrap_or(key);
        let version = spec["version"].as_str().unwrap_or("*");
        let mut cmd = Command::new("cargo");
        cmd.current_dir(root)
            .arg("add")
            .arg(format!("{package}@{version}"));
        if package != key {
            cmd.args(["--rename", key]);
        }
        if spec["default_features"] == Value::Bool(false) {
            cmd.arg("--no-default-features");
        }
        let features: Vec<&str> = spec["features"]
            .as_array()
            .map(|a| a.iter().filter_map(|f| f.as_str()).collect())
            .unwrap_or_default();
        if !features.is_empty() {
            cmd.args(["--features", &features.join(",")]);
        }
        if let Some(target) = spec["target"].as_str() {
            cmd.args(["--target", target]);
        }
        run(cmd, key)?;
    }
    Ok(())
}

fn run(mut cmd: Command, what: &str) -> Result<(), String> {
    let out = cmd.output().map_err(|e| format!("cargo add {what}: {e}"))?;
    if out.status.success() {
        println!("[ok] cargo add {what}");
        Ok(())
    } else {
        Err(format!(
            "cargo add {what} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}
