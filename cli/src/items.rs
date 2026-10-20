//! The public items and drawable types of the library's own source.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, Visibility};

use crate::graph::{Mp, Tree};

pub struct Found {
    pub mp: Mp,
    pub ident: String,
    pub doc: String,
}

pub fn doc_of(attrs: &[syn::Attribute]) -> String {
    for a in attrs {
        if a.path().is_ident("doc")
            && let syn::Meta::NameValue(nv) = &a.meta
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
        {
            let text = s.value();
            let text = text.trim();
            if !text.is_empty() {
                return text.to_string();
            }
        }
    }
    String::new()
}

/// Every public named item, by exact identifier.
pub fn items(tree: &Tree) -> BTreeMap<String, Vec<Found>> {
    let mut out: BTreeMap<String, Vec<Found>> = BTreeMap::new();
    for (mp, m) in &tree.mods {
        for item in &m.ast.items {
            let (ident, vis, attrs) = match item {
                Item::Struct(i) => (i.ident.to_string(), &i.vis, &i.attrs),
                Item::Enum(i) => (i.ident.to_string(), &i.vis, &i.attrs),
                Item::Trait(i) => (i.ident.to_string(), &i.vis, &i.attrs),
                Item::Fn(i) => (i.sig.ident.to_string(), &i.vis, &i.attrs),
                Item::Type(i) => (i.ident.to_string(), &i.vis, &i.attrs),
                Item::Const(i) => (i.ident.to_string(), &i.vis, &i.attrs),
                _ => continue,
            };
            if matches!(vis, Visibility::Public(_)) && !mp.is_empty() {
                out.entry(ident.clone()).or_default().push(Found {
                    mp: mp.clone(),
                    ident,
                    doc: doc_of(attrs),
                });
            }
        }
    }
    // Types a macro defines are still named by the `pub use` that exports them.
    for (mp, m) in &tree.mods {
        for (name, target) in &m.reexports {
            let public = name.starts_with(|c: char| c.is_ascii_uppercase()) && !mp.is_empty();
            let known = out
                .get(name)
                .is_some_and(|v| v.iter().any(|f| &f.mp == target));
            if public && !known && tree.mods.contains_key(target) {
                out.entry(name.clone()).or_default().push(Found {
                    mp: target.clone(),
                    ident: name.clone(),
                    doc: String::new(),
                });
            }
        }
    }
    out
}

pub fn derives_element(attrs: &[syn::Attribute]) -> bool {
    use quote::ToTokens;
    attrs.iter().any(|a| {
        a.path().is_ident("derive") && a.meta.to_token_stream().to_string().contains("IntoElement")
    })
}

/// Public types that draw themselves, reachable from their chapter's root.
pub fn elements(tree: &Tree) -> Vec<(Mp, String, String)> {
    let mut out = vec![];
    for (mp, m) in &tree.mods {
        let Some(chapter) = mp.first() else { continue };
        let root = &tree.mods[&vec![chapter.clone()]];
        let mut drawn: BTreeSet<String> = BTreeSet::new();
        for item in &m.ast.items {
            match item {
                Item::Struct(i) if derives_element(&i.attrs) => {
                    drawn.insert(i.ident.to_string());
                }
                Item::Enum(i) if derives_element(&i.attrs) => {
                    drawn.insert(i.ident.to_string());
                }
                Item::Impl(i) => {
                    if let Some((_, path, _)) = &i.trait_
                        && path
                            .segments
                            .last()
                            .is_some_and(|s| s.ident == "RenderOnce" || s.ident == "Render")
                        && let syn::Type::Path(t) = &*i.self_ty
                        && let Some(seg) = t.path.segments.last()
                    {
                        drawn.insert(seg.ident.to_string());
                    }
                }
                _ => {}
            }
        }
        for item in &m.ast.items {
            let (ident, vis, attrs) = match item {
                Item::Struct(i) if i.generics.params.is_empty() => {
                    (i.ident.to_string(), &i.vis, &i.attrs)
                }
                Item::Enum(i) if i.generics.params.is_empty() => {
                    (i.ident.to_string(), &i.vis, &i.attrs)
                }
                _ => continue,
            };
            let public = matches!(vis, Visibility::Public(_))
                && (root.reexports.contains_key(&ident) || root.defines.contains(&ident));
            if public && drawn.contains(&ident) {
                out.push((mp.clone(), ident, doc_of(attrs)));
            }
        }
    }
    out
}

pub fn sentence(doc: &str) -> String {
    let first = doc.split(". ").next().unwrap_or(doc).trim();
    first.trim_end_matches('.').to_string()
}
