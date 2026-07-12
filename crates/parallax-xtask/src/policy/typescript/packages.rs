use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result};

use super::{TypeScriptProvider, collect_source_files};

pub(crate) fn package_imports(root: &Path) -> Result<BTreeSet<String>> {
    let ui = root.join("ui");
    let provider = TypeScriptProvider::new(&ui.join("tsconfig.json"));
    let mut files = Vec::new();
    collect_source_files(&ui.join("src"), &mut files)?;
    if ui.join("tests").is_dir() {
        collect_source_files(&ui.join("tests"), &mut files)?;
    }
    for config in ["vite.config.ts", "eslint.config.js", "prettier.config.js"] {
        let path = ui.join(config);
        if path.is_file() {
            files.push(path);
        }
    }
    let mut packages = BTreeSet::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        for edge in provider.analyze(&path, &source).imports {
            if let Some(package) = package_name(&edge.specifier) {
                packages.insert(package.to_owned());
            }
        }
    }
    Ok(packages)
}

fn package_name(specifier: &str) -> Option<&str> {
    if specifier.starts_with(['.', '/', '#']) {
        return None;
    }
    if specifier.starts_with('@') {
        let end = specifier
            .match_indices('/')
            .nth(1)
            .map_or(specifier.len(), |(index, _)| index);
        Some(&specifier[..end])
    } else {
        specifier.split('/').next()
    }
}
