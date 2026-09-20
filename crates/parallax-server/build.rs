use std::path::{Component, Path, PathBuf};
use std::{env, fs, process};

const LEGACY_STUB_MARKER: &[u8] = b"parallax embed-ui stub";

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_UI");
    if env::var_os("CARGO_FEATURE_EMBED_UI").is_none() {
        return;
    }

    let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
        eprintln!("CARGO_MANIFEST_DIR is required to locate ui/dist/client");
        process::exit(1);
    };
    let manifest_dir = PathBuf::from(manifest_dir);
    let dist = manifest_dir.join("../../ui/dist/client");
    let shell = dist.join("_shell.html");
    println!("cargo:rerun-if-changed={}", dist.display());
    if let Err(error) = validate_product(&manifest_dir, &dist, &shell) {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn validate_product(manifest_dir: &Path, dist: &Path, shell: &Path) -> Result<(), String> {
    let workspace_root = lexical_normalize(&manifest_dir.join("../.."));
    let product_root = lexical_normalize(dist);
    let canonical_workspace_root = fs::canonicalize(&workspace_root).map_err(|error| {
        format!(
            "cannot resolve workspace root {}: {error}",
            workspace_root.display()
        )
    })?;
    let canonical_product_root = fs::canonicalize(&product_root).map_err(|error| {
        format!(
            "cannot resolve embedded UI product {}: {error}",
            product_root.display()
        )
    })?;
    if !canonical_product_root.starts_with(&canonical_workspace_root) {
        return Err(format!(
            "embedded UI product {} escapes workspace root {}",
            product_root.display(),
            workspace_root.display()
        ));
    }

    reject_symlink_components(&workspace_root, &product_root)?;
    reject_symlink_tree(&product_root)?;

    let bytes = fs::read(shell).map_err(|error| {
        format!(
            "cannot read embedded UI {}: {error}; run `mise run build-ui-for-rust` first",
            shell.display()
        )
    })?;
    if bytes.is_empty() {
        return Err(format!(
            "{} is not a built UI product; run `mise run build-ui-for-rust` first",
            shell.display()
        ));
    }
    if bytes
        .windows(LEGACY_STUB_MARKER.len())
        .any(|window| window == LEGACY_STUB_MARKER)
    {
        return Err(format!(
            "{} contains a legacy embedded UI placeholder; run `mise run build-ui-for-rust` first",
            shell.display()
        ));
    }
    Ok(())
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn reject_symlink_components(root: &Path, path: &Path) -> Result<(), String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        format!(
            "embedded UI product {} is outside workspace root {}: {error}",
            path.display(),
            root.display()
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            format!(
                "cannot inspect embedded UI path {}: {error}",
                current.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "embedded UI path {} contains a symlink; build the product inside the checkout",
                current.display()
            ));
        }
    }
    Ok(())
}

fn reject_symlink_tree(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "cannot inspect embedded UI product {}: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "embedded UI product {} contains a symlink; build the product inside the checkout",
            path.display()
        ));
    }
    if !metadata.is_dir() && !metadata.is_file() {
        return Err(format!(
            "embedded UI product {} contains a special file; only regular files and directories are allowed",
            path.display()
        ));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| {
            format!(
                "cannot inspect embedded UI product {}: {error}",
                path.display()
            )
        })? {
            let entry = entry.map_err(|error| {
                format!(
                    "cannot inspect embedded UI product {}: {error}",
                    path.display()
                )
            })?;
            reject_symlink_tree(&entry.path())?;
        }
    }
    Ok(())
}
