use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail, ensure};
use flate2::read::GzDecoder;
use object::{Architecture, BinaryFormat, Object, ObjectSection, ObjectSymbol, SymbolKind};
use serde_json::Value;
use tar::EntryType;

use super::{VerifySpec, archive};

const MAX_BINARY_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
const OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";

pub(super) fn local(spec: &VerifySpec) -> Result<()> {
    let archive_name = file_name(&spec.archive)?;
    let expected_names = [
        format!("parallax-{}.tar.gz", spec.target),
        format!("parallax-{}-{}.tar.gz", spec.version, spec.target),
    ];
    ensure!(
        expected_names
            .iter()
            .any(|expected| expected == archive_name),
        "archive name `{archive_name}` does not match preview or stable contract"
    );
    let digest = verify_checksum(&spec.archive)?;
    verify_sbom(&spec.archive, archive_name, &digest)?;
    ensure!(
        bundle_path(&spec.archive).is_file(),
        "signature bundle is missing"
    );
    let binary = read_binary(spec)?;
    verify_object(&binary, &spec.target, &spec.version)?;
    Ok(())
}

pub(super) fn signature(spec: &VerifySpec) -> Result<()> {
    run("cosign", &signature_arguments(spec))
}

pub(super) fn provenance(spec: &VerifySpec) -> Result<()> {
    run("gh", &provenance_arguments(spec))
}

fn signature_arguments(spec: &VerifySpec) -> Vec<OsString> {
    vec![
        "verify-blob".into(),
        "--bundle".into(),
        bundle_path(&spec.archive).into_os_string(),
        "--certificate-identity".into(),
        spec.signer_identity.clone().into(),
        "--certificate-oidc-issuer".into(),
        OIDC_ISSUER.into(),
        spec.archive.as_os_str().to_owned(),
    ]
}

fn provenance_arguments(spec: &VerifySpec) -> Vec<OsString> {
    vec![
        "attestation".into(),
        "verify".into(),
        spec.archive.as_os_str().to_owned(),
        "--repo".into(),
        spec.repository.clone().into(),
        "--signer-workflow".into(),
        spec.signer_workflow.clone().into(),
        "--source-digest".into(),
        spec.source_commit.clone().into(),
        "--source-ref".into(),
        spec.source_ref.clone().into(),
        "--deny-self-hosted-runners".into(),
    ]
}

fn verify_checksum(path: &Path) -> Result<String> {
    let checksum_path = sidecar(path, "sha256");
    let expected = std::fs::read_to_string(&checksum_path)
        .with_context(|| format!("read checksum {}", checksum_path.display()))?;
    let expected = expected
        .strip_suffix('\n')
        .context("checksum must end with one newline")?;
    ensure!(
        expected.len() == 64 && expected.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "checksum must contain one SHA-256 hex digest"
    );
    ensure!(
        expected.bytes().all(|byte| !byte.is_ascii_uppercase()),
        "checksum must use lowercase hex"
    );
    let actual = archive::digest(path)?;
    ensure!(expected == actual, "archive checksum mismatch");
    Ok(actual)
}

fn verify_sbom(archive: &Path, archive_name: &str, digest: &str) -> Result<()> {
    let path = sidecar(archive, "sbom.json");
    let document: Value = serde_json::from_slice(
        &std::fs::read(&path).with_context(|| format!("read SBOM {}", path.display()))?,
    )
    .with_context(|| format!("parse CycloneDX SBOM {}", path.display()))?;
    ensure!(document["bomFormat"] == "CycloneDX", "SBOM format mismatch");
    ensure!(document["specVersion"] == "1.6", "SBOM spec mismatch");
    ensure!(
        document["metadata"]["component"]["name"] == archive_name,
        "SBOM archive name mismatch"
    );
    ensure!(
        document["metadata"]["component"]["version"] == format!("sha256:{digest}"),
        "SBOM archive digest mismatch"
    );
    Ok(())
}

fn read_binary(spec: &VerifySpec) -> Result<Vec<u8>> {
    let archive_bytes = std::fs::metadata(&spec.archive)
        .with_context(|| format!("read archive metadata {}", spec.archive.display()))?
        .len();
    ensure!(
        archive_bytes <= MAX_ARCHIVE_BYTES,
        "release archive exceeds 512 MiB"
    );
    let compressed = std::fs::read(&spec.archive)
        .with_context(|| format!("read archive {}", spec.archive.display()))?;
    verify_gzip_header(&compressed)?;
    let binary = {
        let mut archive = tar::Archive::new(GzDecoder::new(compressed.as_slice()));
        let mut entries = archive.entries()?;
        let binary = {
            let mut entry = entries.next().context("archive is empty")??;
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
                spec.source_epoch,
                Some("root".to_string()),
                Some("root".to_string()),
                EntryType::Regular,
            );
            ensure!(actual == expected, "archive metadata mismatch: {actual:?}");
            ensure!(
                header.size()? <= MAX_BINARY_BYTES,
                "archive binary exceeds 512 MiB"
            );
            let mut binary = Vec::with_capacity(usize::try_from(header.size()?)?);
            entry.read_to_end(&mut binary)?;
            binary
        };
        ensure!(
            entries.next().is_none(),
            "archive contains unexpected extra entries"
        );
        binary
    };
    verify_canonical_archive(&compressed, &binary, spec.source_epoch)?;
    Ok(binary)
}

fn verify_canonical_archive(compressed: &[u8], binary: &[u8], source_epoch: u64) -> Result<()> {
    let temporary = tempfile::tempdir().context("create canonical archive verification dir")?;
    let binary_path = temporary.path().join("parallax");
    let archive_path = temporary.path().join("canonical.tar.gz");
    std::fs::write(&binary_path, binary).context("write canonical archive verification binary")?;
    archive::write(&binary_path, &archive_path, source_epoch)?;
    let canonical = std::fs::read(&archive_path).context("read canonical archive verification")?;
    ensure!(
        compressed == canonical,
        "release archive bytes differ from the canonical packager output"
    );
    Ok(())
}

fn verify_gzip_header(compressed: &[u8]) -> Result<()> {
    const HEADER: [u8; 10] = [0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 2, 255];
    ensure!(
        compressed.get(..HEADER.len()) == Some(HEADER.as_slice()),
        "archive gzip header is not deterministic"
    );
    Ok(())
}

pub(super) fn verify_object(binary: &[u8], target: &str, version: &str) -> Result<()> {
    let object = object::File::parse(binary).context("parse release binary object")?;
    let expected = match target {
        "aarch64-apple-darwin" => (BinaryFormat::MachO, Architecture::Aarch64),
        "x86_64-apple-darwin" => (BinaryFormat::MachO, Architecture::X86_64),
        "aarch64-unknown-linux-gnu" => (BinaryFormat::Elf, Architecture::Aarch64),
        "x86_64-unknown-linux-gnu" => (BinaryFormat::Elf, Architecture::X86_64),
        _ => bail!("unsupported release target `{target}`"),
    };
    ensure!(
        (object.format(), object.architecture()) == expected,
        "binary target mismatch: format={:?}, architecture={:?}",
        object.format(),
        object.architecture()
    );
    let sections = object
        .sections()
        .filter_map(|section| section.name().ok())
        .collect::<Vec<_>>();
    ensure!(
        sections.iter().any(|name| is_line_table_section(name)),
        "release binary is missing line tables"
    );
    ensure!(
        object.symbol_table().is_some(),
        "release binary is missing its symbol table"
    );
    verify_symbolication(binary, &object)?;
    let identity = format!("parallax-release-identity:{version}");
    ensure!(
        binary
            .windows(identity.len())
            .any(|window| window == identity.as_bytes()),
        "release binary does not contain expected identity `{identity}`"
    );
    Ok(())
}

fn is_line_table_section(name: &str) -> bool {
    matches!(
        name,
        ".debug_line" | ".zdebug_line" | "__debug_line" | "__zdebug_line"
    )
}

fn verify_symbolication(binary: &[u8], object: &object::File<'_>) -> Result<()> {
    let mut executable = tempfile::NamedTempFile::new().context("create symbolication fixture")?;
    executable
        .write_all(binary)
        .context("write symbolication fixture")?;
    executable.flush().context("flush symbolication fixture")?;
    let loader = addr2line::Loader::new(executable.path())
        .map_err(|error| anyhow!("load release debug data: {error}"))?;
    for symbol in object.symbols().filter(|symbol| {
        symbol.kind() == SymbolKind::Text && symbol.is_definition() && symbol.address() != 0
    }) {
        if loader
            .find_location(symbol.address())
            .map_err(|error| anyhow!("resolve release symbol source location: {error}"))?
            .is_some_and(|location| location.file.is_some() && location.line.is_some())
        {
            return Ok(());
        }
    }
    bail!("release line tables cannot resolve any text symbol to a source line")
}

fn run(program: &str, arguments: &[OsString]) -> Result<()> {
    let status = Command::new(program)
        .args(arguments)
        .status()
        .with_context(|| format!("start {program}"))?;
    ensure!(
        status.success(),
        "{program} verification failed with {status}"
    );
    Ok(())
}

fn file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(OsStr::to_str)
        .context("archive path needs a UTF-8 filename")
}

fn bundle_path(path: &Path) -> PathBuf {
    sidecar(path, "bundle")
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}.{}", path.display(), suffix))
}

#[cfg(test)]
mod tests;
