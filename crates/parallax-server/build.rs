use std::path::PathBuf;
use std::{env, fs, process};

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_UI");
    if env::var_os("CARGO_FEATURE_EMBED_UI").is_none() {
        return;
    }

    let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
        eprintln!("CARGO_MANIFEST_DIR is required to locate ui/dist/client");
        process::exit(1);
    };
    let dist = PathBuf::from(manifest_dir).join("../../ui/dist/client");
    let shell = dist.join("_shell.html");
    println!("cargo:rerun-if-changed={}", dist.display());
    if shell.is_file() {
        return;
    }

    if let Err(error) = fs::create_dir_all(&dist) {
        eprintln!("failed to create {}: {error}", dist.display());
        process::exit(1);
    }
    if let Err(error) = fs::write(
        &shell,
        b"<!doctype html><title>parallax embed-ui stub</title>\n",
    ) {
        eprintln!("failed to write {}: {error}", shell.display());
        process::exit(1);
    }
}
