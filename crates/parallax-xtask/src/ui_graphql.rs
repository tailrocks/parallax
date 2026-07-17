//! Plan 152 GraphQL SDL export / drift check / contract orchestration.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as Process,
};

use anyhow::{Context, Result, bail, ensure};
use tempfile::TempDir;

const SCHEMA_REL: &str = "ui/graphql/schema.graphql";
const CODEGEN_SCRIPT: &str = "graphql:generate";

/// Export the authoritative SDL from `parallax_api::export_schema_sdl`.
pub(crate) fn export(root: &Path) -> Result<()> {
    println!("==> graphql: exporting schema SDL from parallax-api");
    let sdl = parallax_api::export_schema_sdl();
    ensure!(!sdl.trim().is_empty(), "exported SDL is empty");
    let path = root.join(SCHEMA_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    atomic_write(&path, sdl.as_bytes())?;
    println!("==> graphql: wrote {SCHEMA_REL} ({} bytes)", sdl.len());
    Ok(())
}

/// Check schema drift plus optional codegen artifact drift when configured.
pub(crate) fn check(root: &Path) -> Result<()> {
    println!("==> graphql: checking schema SDL drift");
    let expected = parallax_api::export_schema_sdl();
    ensure!(!expected.trim().is_empty(), "exported SDL is empty");

    let path = root.join(SCHEMA_REL);
    ensure!(
        path.is_file(),
        "missing checked-in schema at {SCHEMA_REL}; run `cargo xtask ui graphql export`"
    );
    let on_disk =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    // Check mode must never modify the worktree: compare in memory only.
    if on_disk != expected {
        bail!(
            "GraphQL schema drift: {SCHEMA_REL} differs from parallax_api::export_schema_sdl(); run `cargo xtask ui graphql export`"
        );
    }
    println!("==> graphql: schema SDL is current");

    // When codegen is wired (package script present), verify generate-format is
    // deterministic. Before packages land, schema-only check is sufficient.
    if ui_has_graphql_generate(root)? {
        check_codegen_reproducible(root)?;
    }
    Ok(())
}

fn ui_has_graphql_generate(root: &Path) -> Result<bool> {
    let package_json = root.join("ui/package.json");
    if !package_json.is_file() {
        return Ok(false);
    }
    let text = fs::read_to_string(&package_json)?;
    Ok(text.contains(&format!("\"{CODEGEN_SCRIPT}\"")))
}

fn check_codegen_reproducible(root: &Path) -> Result<()> {
    println!("==> graphql: verifying codegen is reproducible");
    let ui = root.join("ui");
    let tracked = collect_generated_artifacts(&ui)?;
    ensure!(
        !tracked.is_empty(),
        "graphql:generate is configured but no generated artifacts were found under ui/src"
    );

    let before = snapshot_files(&tracked)?;
    run_ui_script(&ui, CODEGEN_SCRIPT)?;
    let after = snapshot_files(&tracked)?;
    if before != after {
        // Restore is the operator's job after fix; fail closed on drift.
        bail!(
            "GraphQL codegen drift: re-running `bun run {CODEGEN_SCRIPT}` changed generated artifacts; commit the regenerated output or fix the generator"
        );
    }
    println!("==> graphql: codegen artifacts are byte-stable");
    Ok(())
}

fn collect_generated_artifacts(ui: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let candidates = [
        ui.join("src/platform/graphql/generated"),
        ui.join("src/platform/graphql/tests/fixtures"),
        ui.join("src/features"),
    ];
    for root in candidates {
        if root.is_dir() {
            walk_generated(&root, &mut files)?;
        }
    }
    files.sort();
    Ok(files)
}

fn walk_generated(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_generated(&path, files)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".generated.ts"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn snapshot_files(files: &[PathBuf]) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut out = Vec::with_capacity(files.len());
    for path in files {
        let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        out.push((path.clone(), bytes));
    }
    Ok(out)
}

fn run_ui_script(ui: &Path, script: &str) -> Result<()> {
    println!("==> bun run {script}");
    let status = Process::new("bun")
        .args(["run", script])
        .current_dir(ui)
        .status()
        .with_context(|| format!("failed to start bun run {script}"))?;
    ensure!(status.success(), "bun run {script} failed with {status}");
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("schema path must have a parent directory")?;
    let temp_dir = TempDir::new_in(parent)
        .with_context(|| format!("failed to create temp dir under {}", parent.display()))?;
    let temp_path = temp_dir.path().join("schema.graphql.tmp");
    fs::write(&temp_path, bytes)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn normalize_round_trip_matches_export() {
        let sdl = parallax_api::export_schema_sdl();
        assert_eq!(parallax_api::normalize_schema_sdl(&sdl), sdl);
        assert!(sdl.contains("type Query"));
    }

    #[test]
    fn export_writes_and_check_is_clean() {
        let root = TempDir::new().expect("temp");
        let schema_dir = root.path().join("ui/graphql");
        fs::create_dir_all(&schema_dir).expect("mkdir");
        // Minimal package.json without codegen script → schema-only check.
        fs::create_dir_all(root.path().join("ui")).expect("ui");
        fs::write(
            root.path().join("ui/package.json"),
            r#"{"name":"ui","scripts":{}}"#,
        )
        .expect("package");

        export(root.path()).expect("export");
        let written = fs::read_to_string(root.path().join(SCHEMA_REL)).expect("read");
        assert_eq!(written, parallax_api::export_schema_sdl());
        check(root.path()).expect("check clean");

        // Tamper → check fails and does not rewrite.
        fs::write(root.path().join(SCHEMA_REL), "schema { query: Query }\n").expect("tamper");
        let err = check(root.path()).expect_err("drift must fail");
        assert!(
            err.to_string().contains("schema drift"),
            "unexpected error: {err:#}"
        );
        let after = fs::read_to_string(root.path().join(SCHEMA_REL)).expect("read after");
        assert_eq!(after, "schema { query: Query }\n");
    }
}
