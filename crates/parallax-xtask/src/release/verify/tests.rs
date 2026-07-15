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
    std::fs::write(
        sidecar(&archive_path, "sha256"),
        digest.to_ascii_uppercase(),
    )?;
    let uppercase_without_newline = verify_checksum(&archive_path).is_err();
    std::fs::write(
        sidecar(&archive_path, "sha256"),
        format!("{digest}\n{digest}\n"),
    )?;
    let multiline_checksum = verify_checksum(&archive_path).is_err();
    archive::write_checksum(&archive_path)?;
    write_sbom(&sbom_path, "wrong.tar.gz", &digest)?;
    let bad_sbom_name = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();
    write_sbom(&sbom_path, file_name(&archive_path)?, &"0".repeat(64))?;
    let bad_sbom_digest = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();
    std::fs::write(
        &sbom_path,
        br#"{"bomFormat":"SPDX","specVersion":"1.6","metadata":{}}"#,
    )?;
    let bad_sbom_format = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();
    std::fs::write(&sbom_path, b"not-json")?;
    let malformed_sbom = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();
    std::fs::remove_file(&sbom_path)?;
    let missing_sbom = verify_sbom(&archive_path, file_name(&archive_path)?, &digest).is_err();

    let actual = (
        valid,
        bad_checksum,
        uppercase_without_newline,
        multiline_checksum,
        bad_sbom_name,
        bad_sbom_digest,
        bad_sbom_format,
        malformed_sbom,
        missing_sbom,
    );
    let expected = ((true, true), true, true, true, true, true, true, true, true);
    if actual != expected {
        return Err(format!("release sidecar verification mismatch: {actual:?}").into());
    }
    Ok(())
}

#[test]
fn archive_layout_tampering_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let target = host_target()?;
    let cases = [
        ("wrong-path", "bin/parallax", 0o755, 1_700_000_000, false),
        ("wrong-mode", "parallax", 0o644, 1_700_000_000, false),
        ("wrong-owner-time", "parallax", 0o755, 1_700_000_001, false),
        ("extra-entry", "parallax", 0o755, 1_700_000_000, true),
    ];
    for (name, path, mode, mtime, extra) in cases {
        let archive_path = temp.path().join(format!("{name}.tar.gz"));
        write_archive_fixture(&archive_path, path, mode, mtime, extra)?;
        let spec = spec(archive_path, target, env!("CARGO_PKG_VERSION"));
        if read_binary(&spec).is_ok() {
            return Err(format!("tampered archive `{name}` unexpectedly passed").into());
        }
    }
    Ok(())
}

fn write_archive_fixture(
    path: &Path,
    entry_path: &str,
    mode: u32,
    mtime: u64,
    extra: bool,
) -> Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::{Builder, EntryType, Header};

    let encoder = GzEncoder::new(std::fs::File::create(path)?, Compression::default());
    let mut builder = Builder::new(encoder);
    {
        let mut append = |name: &str| -> Result<()> {
            let payload = b"fixture";
            let mut header = Header::new_gnu();
            header.set_path(name)?;
            header.set_mode(mode);
            header.set_uid(0);
            header.set_gid(0);
            header.set_mtime(mtime);
            header.set_username("root")?;
            header.set_groupname("root")?;
            header.set_entry_type(EntryType::Regular);
            header.set_size(payload.len() as u64);
            header.set_cksum();
            builder.append(&header, payload.as_slice())?;
            Ok(())
        };
        append(entry_path)?;
        if extra {
            append("unexpected")?;
        }
    }
    builder.finish()?;
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
