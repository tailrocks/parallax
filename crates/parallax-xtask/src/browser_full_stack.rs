//! Browser full-stack lifecycle orchestration (plan 145).
//!
//! Xtask starts the locked example harness. Mode selection (attach vs managed)
//! and OTLP seed/readiness live in the example.

use std::{path::Path, process::Command};

use anyhow::{Context, Result, bail};

pub(crate) fn run(root: &Path) -> Result<()> {
    println!("==> browser full-stack harness via parallax-server example");
    let status = Command::new("cargo")
        .args([
            "run",
            "--locked",
            "-p",
            "parallax-server",
            "--example",
            "browser_full_stack_serve",
            "--quiet",
        ])
        .current_dir(root)
        .status()
        .context("start browser_full_stack_serve example")?;
    if !status.success() {
        bail!("browser_full_stack_serve exited with {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_source_exists() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(
            root.join("crates/parallax-server/examples/browser_full_stack_serve.rs")
                .is_file()
        );
    }
}
