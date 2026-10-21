//! `cargo mok update`: copied files follow the source, and the project's edits stay.

use std::path::Path;

use crate::install::{report, sync};
use crate::lock::Lock;
use crate::source::{Options, load, revision, source_root};
use crate::sync::Action;

pub fn update(root: &Path, opts: &Options, dry: bool) -> Result<(), String> {
    let lock = Lock::read(root, &opts.dir.to_string_lossy());
    if lock.files.is_empty() {
        return Err("nothing was copied yet: run `cargo mok add <component>` first".into());
    }
    let source = source_root(opts.from.as_deref())?;
    let reg = load(&source)?;
    let (kept, gone): (Vec<_>, Vec<_>) = lock
        .units()
        .into_iter()
        .partition(|unit| reg.units.contains_key(unit));
    for unit in &gone {
        println!("[mok] {unit} is gone from the source; its file stays as it is");
    }
    let now = revision(&source);
    match (lock.source.as_str(), now.as_str()) {
        (from, to) if !from.is_empty() && !to.is_empty() => println!("[mok] {from} -> {to}"),
        (_, to) if !to.is_empty() => println!("[mok] now {to}"),
        _ => {}
    }
    let wanted = kept.into_iter().collect();
    let rows = sync(root, &source, &reg, lock, &wanted, opts, dry)?;
    report(&rows, dry);
    let conflicts = rows.iter().filter(|(_, a)| *a == Action::Conflict).count();
    if conflicts > 0 {
        return Err(format!(
            "{conflicts} file(s) hold conflicts: fix the marks between <<<<<<< and >>>>>>>, then commit"
        ));
    }
    Ok(())
}
