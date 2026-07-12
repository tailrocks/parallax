use super::*;

fn write_fixture(root: &Path, version: &str) {
    fs::write(
        root.join("rust-toolchain.toml"),
        format!(
            "[toolchain]\nchannel = '{version}'\nprofile = 'minimal'\ncomponents = ['rustfmt', 'clippy']\ntargets = ['aarch64-apple-darwin', 'x86_64-apple-darwin', 'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnu']\n"
        ),
    )
    .expect("toolchain fixture");
    fs::write(
        root.join("mise.toml"),
        format!("[tools]\nrust = '{version}'\n"),
    )
    .expect("mise fixture");
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace.package]\nrust-version = '{version}'\nedition = '2024'\n"),
    )
    .expect("Cargo fixture");
}

#[test]
fn toolchain_files_require_exact_agreement_and_inventory() {
    let directory = tempfile::tempdir().expect("temporary directory");
    write_fixture(directory.path(), RUST_VERSION);
    let mut findings = Vec::new();
    check_files(directory.path(), &mut findings).expect("positive fixture");
    assert!(findings.is_empty());

    write_fixture(directory.path(), "stable");
    check_files(directory.path(), &mut findings).expect("negative fixture");
    assert!(findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "product.rust-toolchain" | "product.rust-toolchain-agreement"
        )
    }));
}
