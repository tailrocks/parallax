use super::*;

#[test]
fn recognizes_native_and_compressed_line_tables() -> Result<(), String> {
    let actual = [
        ".debug_line",
        ".zdebug_line",
        "__debug_line",
        "__zdebug_line",
    ]
    .map(is_line_table_section);
    if actual != [true, true, true, true] || is_line_table_section(".debug_info") {
        return Err(format!(
            "line-table section recognition mismatch: {actual:?}"
        ));
    }
    Ok(())
}

#[test]
fn local_verification_rejects_metadata_version_and_missing_bundle()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let target = host_target()?;
    let archive_path = temp.path().join(format!("parallax-{target}.tar.gz"));
    let fixture_binary = temp.path().join("parallax");
    std::fs::write(&fixture_binary, b"tiny fixture")?;
    archive::write(&fixture_binary, &archive_path, 1_700_000_000)?;
    let digest = archive::write_checksum(&archive_path)?;
    write_sbom(
        &sidecar(&archive_path, "sbom.json"),
        file_name(&archive_path)?,
        &digest,
    )?;
    let mut spec = spec(archive_path, target, env!("CARGO_PKG_VERSION"));
    let missing_bundle = local(&spec).is_err();
    spec.source_epoch += 1;
    let bad_epoch = read_binary(&spec).is_err();

    let actual = (bad_epoch, missing_bundle);
    if actual != (true, true) {
        return Err(format!("local release verification mismatch: {actual:?}").into());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn object_verification_rejects_target_version_and_corrupt_line_tables()
-> Result<(), Box<dyn std::error::Error>> {
    let target = host_target()?;
    let mut binary = std::fs::read(std::env::current_exe()?)?;
    binary.extend_from_slice(
        format!("parallax-release-identity:{}", env!("CARGO_PKG_VERSION")).as_bytes(),
    );
    let valid = verify_object(&binary, target, env!("CARGO_PKG_VERSION")).is_ok();
    let bad_version = verify_object(&binary, target, "999.999.999-identity-missing").is_err();
    let bad_target =
        verify_object(&binary, alternate_target(target), env!("CARGO_PKG_VERSION")).is_err();
    let mut corrupt_debug = binary;
    let line_range = {
        let object = object::File::parse(corrupt_debug.as_slice())?;
        object
            .sections()
            .find(|section| section.name().is_ok_and(is_line_table_section))
            .and_then(|section| section.file_range())
            .ok_or("debug line section has no file range")?
    };
    let start = usize::try_from(line_range.0)?;
    let end = start + usize::try_from(line_range.1)?;
    corrupt_debug[start..end].fill(0);
    let bad_debug = verify_object(&corrupt_debug, target, env!("CARGO_PKG_VERSION")).is_err();

    let actual = (valid, bad_debug, bad_target, bad_version);
    if actual != (true, true, true, true) {
        return Err(format!("release object verification mismatch: {actual:?}").into());
    }
    Ok(())
}

fn alternate_target(target: &str) -> &'static str {
    match target {
        "aarch64-unknown-linux-gnu" => "x86_64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu" => "aarch64-unknown-linux-gnu",
        "aarch64-apple-darwin" => "x86_64-apple-darwin",
        "x86_64-apple-darwin" => "aarch64-apple-darwin",
        _ => unreachable!("host_target returns a supported target"),
    }
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
