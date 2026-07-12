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
    let mut cargo = format!(
        "[workspace.package]\nrust-version = '{version}'\nedition = '2024'\n[profile.release]\ndebug = 'line-tables-only'\nstrip = 'none'\n[workspace.lints.rust]\nunsafe_code = 'forbid'\n"
    );
    for name in RUST_WARN {
        cargo.push_str(&format!("{name} = 'warn'\n"));
    }
    cargo.push_str("[workspace.lints.rustdoc]\nbroken_intra_doc_links = 'deny'\n");
    for name in RUSTDOC_WARN {
        cargo.push_str(&format!("{name} = 'warn'\n"));
    }
    cargo.push_str("[workspace.lints.clippy]\n");
    for name in CLIPPY_WARN {
        cargo.push_str(&format!("{name} = 'warn'\n"));
    }
    fs::write(root.join("Cargo.toml"), cargo).expect("Cargo fixture");
    fs::write(
        root.join("clippy.toml"),
        "too-many-lines-threshold=100\ncognitive-complexity-threshold=25\ntoo-many-arguments-threshold=6\nexcessive-nesting-threshold=4\ndisallowed-methods=[{path='std::thread::sleep'},{path='reqwest::blocking::get'}]\n",
    )
    .expect("Clippy fixture");
}

#[test]
fn toolchain_files_require_exact_agreement_and_inventory() -> Result<()> {
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

    write_fixture(directory.path(), RUST_VERSION);
    let cargo_path = directory.path().join("Cargo.toml");
    let stripped = fs::read_to_string(&cargo_path)
        .expect("read Cargo fixture")
        .replace("debug = 'line-tables-only'", "debug = false")
        .replace("strip = 'none'", "strip = 'debuginfo'");
    fs::write(&cargo_path, stripped).expect("stripped Cargo fixture");
    let mut stripped_findings = Vec::new();
    check_files(directory.path(), &mut stripped_findings).expect("stripped fixture");
    anyhow::ensure!(
        stripped_findings
            .iter()
            .any(|finding| finding.rule_id == "product.release-line-tables"),
        "stripped release profile was accepted"
    );

    write_fixture(directory.path(), RUST_VERSION);
    let weakened = fs::read_to_string(&cargo_path)
        .expect("read lint fixture")
        .replace("pedantic = 'warn'", "pedantic = 'allow'");
    fs::write(&cargo_path, weakened).expect("weakened lint fixture");
    let mut weakened_findings = Vec::new();
    check_files(directory.path(), &mut weakened_findings).expect("weakened fixture");
    anyhow::ensure!(
        weakened_findings
            .iter()
            .any(|finding| finding.rule_id == "product.rust-lint-matrix"),
        "weakened lint matrix was accepted"
    );
    Ok(())
}
