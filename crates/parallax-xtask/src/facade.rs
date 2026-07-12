use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use syn::{Attribute, Item, Visibility};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Manifest {
    schema_version: u32,
    roots: BTreeMap<String, Vec<String>>,
}

pub fn refresh(root: &Path) -> Result<()> {
    for (crate_dir, manifest) in collect(root)? {
        let output = toml::to_string_pretty(&manifest)?;
        fs::write(crate_dir.join("facade.toml"), output)
            .with_context(|| format!("failed to write facade for {}", crate_dir.display()))?;
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<()> {
    let mut drift = Vec::new();
    for (crate_dir, expected) in collect(root)? {
        let path = crate_dir.join("facade.toml");
        let source = fs::read_to_string(&path).with_context(|| {
            format!(
                "missing facade manifest {}; run `cargo xtask facade refresh`",
                path.display()
            )
        })?;
        let actual: Manifest = toml::from_str(&source)
            .with_context(|| format!("malformed facade manifest {}", path.display()))?;
        if actual != expected {
            drift.push(path);
        }
    }
    if !drift.is_empty() {
        bail!(
            "facade drift in {}; run `cargo xtask facade refresh`",
            drift
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

fn collect(root: &Path) -> Result<Vec<(PathBuf, Manifest)>> {
    let mut manifests = Vec::new();
    for entry in fs::read_dir(root.join("crates"))? {
        let crate_dir = entry?.path();
        if !crate_dir.is_dir() || !crate_dir.join("Cargo.toml").is_file() {
            continue;
        }
        let mut roots = BTreeMap::new();
        for name in ["lib.rs", "main.rs"] {
            let path = crate_dir.join("src").join(name);
            if path.is_file() {
                roots.insert(name.into(), parse_root(&path)?);
            }
        }
        manifests.push((
            crate_dir,
            Manifest {
                schema_version: 1,
                roots,
            },
        ));
    }
    manifests.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(manifests)
}

fn parse_root(path: &Path) -> Result<Vec<String>> {
    let source = fs::read_to_string(path)?;
    let syntax =
        syn::parse_file(&source).with_context(|| format!("failed to parse {}", path.display()))?;
    let mut entries = Vec::new();
    for item in syntax.items {
        let (visibility, attributes, kind) = match &item {
            Item::Mod(item) => (&item.vis, &item.attrs, format!("mod {}", item.ident)),
            Item::Use(item) => (
                &item.vis,
                &item.attrs,
                format!("use {}", compact(item.tree.to_token_stream().to_string())),
            ),
            Item::Const(item) => (&item.vis, &item.attrs, format!("const {}", item.ident)),
            Item::Enum(item) => (&item.vis, &item.attrs, format!("enum {}", item.ident)),
            Item::Fn(item) => (&item.vis, &item.attrs, format!("fn {}", item.sig.ident)),
            Item::Static(item) => (&item.vis, &item.attrs, format!("static {}", item.ident)),
            Item::Struct(item) => (&item.vis, &item.attrs, format!("struct {}", item.ident)),
            Item::Trait(item) => (&item.vis, &item.attrs, format!("trait {}", item.ident)),
            Item::Type(item) => (&item.vis, &item.attrs, format!("type {}", item.ident)),
            _ => continue,
        };
        if matches!(visibility, Visibility::Public(_)) {
            let cfg = cfg(attributes);
            entries.push(if cfg.is_empty() {
                kind
            } else {
                format!("{cfg} {kind}")
            });
        }
    }
    entries.sort();
    Ok(entries)
}

fn cfg(attributes: &[Attribute]) -> String {
    let mut values: Vec<_> = attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("cfg"))
        .map(|attribute| compact(attribute.meta.to_token_stream().to_string()))
        .collect();
    values.sort();
    values.join(" ")
}

fn compact(tokens: String) -> String {
    tokens
        .replace(" :: ", "::")
        .replace("{ ", "{")
        .replace(" { ", "{")
        .replace(" }", "}")
        .replace(" , ", ", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn captures_sorted_cfg_and_nested_reexports() {
        let path = std::env::temp_dir().join(format!(
            "facade-{}.rs",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::write(
            &path,
            "pub use crate::{b::{C, D}, a};\n#[cfg(feature = \"x\")] pub mod gated;\nmod private;",
        )
        .expect("fixture write");
        let entries = parse_root(&path).expect("fixture parse");
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("cfg"));
        assert!(
            entries[1].contains("b::{C") && entries[1].contains("D}"),
            "{entries:?}"
        );
        fs::remove_file(path).expect("fixture remove");
    }

    #[test]
    fn malformed_root_fails_closed() {
        let path = std::env::temp_dir().join("facade-malformed.rs");
        fs::write(&path, "pub mod {").expect("fixture write");
        assert!(parse_root(&path).is_err());
        fs::remove_file(path).expect("fixture remove");
    }
}
