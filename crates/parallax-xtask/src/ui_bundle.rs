//! Plan 148 — UI production bundle analysis and two-clean-build determinism.

use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

pub(crate) fn analyze(root: &Path) -> Result<()> {
    run_bun_script(root, &["run", "scripts/bundle-analyze.ts", "--check"])
}

pub(crate) fn build_twice(root: &Path) -> Result<()> {
    run_bun_script(root, &["run", "scripts/bundle-analyze.ts", "--build-twice"])
}

fn run_bun_script(root: &Path, args: &[&str]) -> Result<()> {
    let ui = root.join("ui");
    println!("==> bun {}", args.join(" "));
    let status = Command::new("bun")
        .args(args)
        .current_dir(&ui)
        .stdin(Stdio::null())
        .status()
        .with_context(|| format!("failed to start bun in {}", ui.display()))?;
    if !status.success() {
        bail!("bun {} exited with {status}", args.join(" "));
    }
    Ok(())
}
