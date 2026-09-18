use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_UI");
    println!("cargo:rerun-if-changed=../../mise.toml");
    println!("cargo:rerun-if-changed=../../ui/package.json");
    println!("cargo:rerun-if-changed=../../ui/bun.lock");
    println!("cargo:rerun-if-changed=../../ui/bunfig.toml");
    println!("cargo:rerun-if-changed=../../ui/tsconfig.json");
    println!("cargo:rerun-if-changed=../../ui/vite.config.ts");
    println!("cargo:rerun-if-changed=../../ui/codegen.ts");
    println!("cargo:rerun-if-changed=../../ui/src");
    println!("cargo:rerun-if-changed=../../ui/public");
    println!("cargo:rerun-if-changed=../../ui/scripts");
    println!("cargo:rerun-if-changed=../../ui/graphql");

    if std::env::var_os("CARGO_FEATURE_EMBED_UI").is_none() {
        return;
    }

    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("Cargo must set CARGO_MANIFEST_DIR for the UI build"),
    );
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("parallax-server must remain under the workspace crates directory");

    run_bun(workspace_root, &["install", "--frozen-lockfile"]);
    run_bun(workspace_root, &["run", "build"]);

    let embedded_shell = workspace_root.join("ui/dist/client/_shell.html");
    assert!(
        embedded_shell.is_file(),
        "UI build completed without {}",
        embedded_shell.display()
    );
}

fn run_bun(workspace_root: &Path, arguments: &[&str]) {
    let status = Command::new("bun")
        .current_dir(workspace_root.join("ui"))
        .args(arguments)
        .status()
        .expect("failed to start Bun for the embedded UI build");
    assert!(
        status.success(),
        "Bun UI command failed with status {status}"
    );
}
