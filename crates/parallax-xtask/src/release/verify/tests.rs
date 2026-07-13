use super::*;

#[test]
fn local_verification_rejects_metadata_version_and_missing_bundle()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let target = host_target()?;
    let archive_path = temp.path().join(format!("parallax-{target}.tar.gz"));
    let fixture_binary = temp.path().join("parallax");
    let mut binary = std::fs::read(std::env::current_exe()?)?;
    binary.extend_from_slice(
        format!("parallax-release-identity:{}", env!("CARGO_PKG_VERSION")).as_bytes(),
    );
    std::fs::write(&fixture_binary, binary)?;
    archive::write(&fixture_binary, &archive_path, 1_700_000_000)?;
    let digest = archive::write_checksum(&archive_path)?;
    write_sbom(
        &sidecar(&archive_path, "sbom.json"),
        file_name(&archive_path)?,
        &digest,
    )?;
    std::fs::write(bundle_path(&archive_path), b"bundle fixture")?;
    let mut spec = spec(archive_path.clone(), target, env!("CARGO_PKG_VERSION"));
    let valid = local(&spec).is_ok();
    let mut corrupt_debug = std::fs::read(&fixture_binary)?;
    let line_range = {
        let object = object::File::parse(corrupt_debug.as_slice())?;
        object
            .sections()
            .find(|section| {
                section
                    .name()
                    .is_ok_and(|name| matches!(name, ".debug_line" | "__debug_line"))
            })
            .and_then(|section| section.file_range())
            .ok_or("debug line section has no file range")?
    };
    let start = usize::try_from(line_range.0)?;
    let end = start + usize::try_from(line_range.1)?;
    corrupt_debug[start..end].fill(0);
    let bad_debug = verify_object(&corrupt_debug, target, env!("CARGO_PKG_VERSION")).is_err();
    spec.source_epoch += 1;
    let bad_epoch = local(&spec).is_err();
    spec.source_epoch -= 1;
    spec.version = "999.999.999-identity-missing".to_string();
    let bad_version = local(&spec).is_err();
    spec.version = env!("CARGO_PKG_VERSION").to_string();
    std::fs::remove_file(bundle_path(&archive_path))?;
    let missing_bundle = local(&spec).is_err();

    let actual = (valid, bad_debug, bad_epoch, bad_version, missing_bundle);
    if actual != (true, true, true, true, true) {
        return Err(format!("local release verification mismatch: {actual:?}").into());
    }
    Ok(())
}

#[test]
fn checksum_and_sbom_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let archive_path = temp
        .path()
        .join("parallax-aarch64-unknown-linux-gnu.tar.gz");
    std::fs::write(&archive_path, b"archive")?;
    let digest = archive::write_checksum(&archive_path)?;
    let sbom_path = sidecar(&archive_path, "sbom.json");
    write_sbom(&sbom_path, file_name(&archive_path)?, &digest)?;
    let valid = (
        verify_checksum(&archive_path).is_ok(),
        verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_ok(),
    );

    std::fs::write(
        sidecar(&archive_path, "sha256"),
        format!("{}\n", "0".repeat(64)),
    )?;
    let bad_checksum = verify_checksum(&archive_path).is_err();
    archive::write_checksum(&archive_path)?;
    write_sbom(&sbom_path, "wrong.tar.gz", &digest)?;
    let bad_sbom = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();
    std::fs::remove_file(&sbom_path)?;
    let missing_sbom = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();

    let actual = (valid, bad_checksum, bad_sbom, missing_sbom);
    let expected = ((true, true), true, true, true);
    if actual != expected {
        return Err(format!("release sidecar verification mismatch: {actual:?}").into());
    }
    Ok(())
}

fn write_sbom(path: &Path, name: &str, digest: &str) -> Result<()> {
    std::fs::write(
        path,
        serde_json::to_vec(&serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.6",
            "metadata": {"component": {"name": name, "version": format!("sha256:{digest}")}}
        }))?,
    )?;
    Ok(())
}

fn spec(archive: PathBuf, target: &str, version: &str) -> VerifySpec {
    VerifySpec {
        archive,
        target: target.to_string(),
        version: version.to_string(),
        source_epoch: 1_700_000_000,
        source_commit: "a".repeat(40),
        source_ref: "refs/heads/main".to_string(),
        repository: "tailrocks/parallax".to_string(),
        signer_identity:
            "https://github.com/tailrocks/parallax/.github/workflows/preview.yml@refs/heads/main"
                .to_string(),
        signer_workflow: "tailrocks/parallax/.github/workflows/preview.yml".to_string(),
    }
}

fn host_target() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        pair => Err(format!("unsupported test host {pair:?}")),
    }
}
