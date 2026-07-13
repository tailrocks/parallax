use std::io::Read;

use flate2::read::GzDecoder;
use tar::EntryType;

use super::*;

#[test]
fn deterministic_archive_has_exact_public_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let binary = temp.path().join("input-parallax");
    std::fs::write(&binary, b"fixture-binary")?;
    let first = temp.path().join("first.tar.gz");
    let second = temp.path().join("second.tar.gz");
    archive::write(&binary, &first, 1_700_000_000)?;
    archive::write(&binary, &second, 1_700_000_000)?;
    let first_bytes = std::fs::read(&first)?;
    let second_bytes = std::fs::read(&second)?;
    if first_bytes != second_bytes {
        return Err("identical inputs produced different archives".into());
    }
    if first_bytes[..10] != [0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 2, 255] {
        return Err(format!("gzip header drifted: {:x?}", &first_bytes[..10]).into());
    }

    let mut tar = tar::Archive::new(GzDecoder::new(first_bytes.as_slice()));
    let mut entries = tar.entries()?;
    let mut entry = entries.next().ok_or("archive entry missing")??;
    let header = entry.header();
    let actual = (
        entry.path()?.to_string_lossy().into_owned(),
        header.mode()?,
        header.uid()?,
        header.gid()?,
        header.mtime()?,
        header.username()?.map(str::to_owned),
        header.groupname()?.map(str::to_owned),
        header.entry_type(),
    );
    let expected = (
        "parallax".to_string(),
        0o755,
        0,
        0,
        1_700_000_000,
        Some("root".to_string()),
        Some("root".to_string()),
        EntryType::Regular,
    );
    if actual != expected {
        return Err(format!("archive contract mismatch: {actual:?}").into());
    }
    let mut payload = Vec::new();
    entry.read_to_end(&mut payload)?;
    if payload != b"fixture-binary" {
        return Err("archive payload changed".into());
    }
    if entries.next().is_some() {
        return Err("archive contains more than one entry".into());
    }
    Ok(())
}

#[test]
fn identity_rejects_unsupported_targets_and_ambiguous_versions() -> Result<(), String> {
    let actual = (
        validate_identity("x86_64-unknown-linux-gnu", "0.1.0").is_ok(),
        validate_identity("powerpc-unknown-linux-gnu", "0.1.0").is_err(),
        validate_identity("x86_64-unknown-linux-gnu", "v0.1.0").is_err(),
        validate_identity("x86_64-unknown-linux-gnu", "0.1.0 bad").is_err(),
    );
    if actual != (true, true, true, true) {
        return Err(format!("release identity validation mismatch: {actual:?}"));
    }
    Ok(())
}
