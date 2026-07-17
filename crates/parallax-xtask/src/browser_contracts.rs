//! Browser product-contract server lifecycle orchestration (plan 144).
//!
//! Xtask only starts the locked example harness; seed/reset live in
//! `parallax-test-support` and the example injects the in-memory adapter at the
//! server composition seam.

use std::{path::Path, process::Command};

use anyhow::{Context, Result, bail};

pub(crate) fn run(root: &Path) -> Result<()> {
    println!("==> browser contracts harness via parallax-server example");
    let status = Command::new("cargo")
        .args([
            "run",
            "--locked",
            "-p",
            "parallax-server",
            "--example",
            "browser_contracts_serve",
            "--quiet",
        ])
        .current_dir(root)
        .status()
        .context("start browser_contracts_serve example")?;
    if !status.success() {
        bail!("browser_contracts_serve exited with {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_source_exists() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(
            root.join("crates/parallax-server/examples/browser_contracts_serve.rs")
                .is_file()
        );
    }
}
