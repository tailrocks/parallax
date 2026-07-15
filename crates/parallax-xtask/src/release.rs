use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;

mod archive;
mod verify;

const TARGETS: [&str; 4] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
];
const MAX_RELEASE_BINARY_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum Channel {
    Preview,
    Stable,
    Rehearsal,
}

#[derive(Debug)]
pub(crate) struct VerifySpec {
    pub archive: PathBuf,
    pub target: String,
    pub version: String,
    pub source_epoch: u64,
    pub source_commit: String,
    pub source_ref: String,
    pub repository: String,
    pub signer_identity: String,
    pub signer_workflow: String,
}

pub(crate) fn package(
    binary: &Path,
    output: &Path,
    target: &str,
    version: &str,
    channel: Channel,
    source_epoch: u64,
) -> Result<()> {
    validate_identity(target, version)?;
    validate_channel_version(version, channel)?;
    validate_archive_name(output, target, version, channel)?;
    validate_binary(binary, target, version)?;
    println!(
        "==> package {} -> {} (SOURCE_DATE_EPOCH={source_epoch})",
        binary.display(),
        output.display()
    );
    archive::write(binary, output, source_epoch)?;
    let digest = archive::write_checksum(output)?;
    println!("==> archive sha256 {digest}");
    Ok(())
}

fn validate_archive_name(
    output: &Path,
    target: &str,
    version: &str,
    channel: Channel,
) -> Result<()> {
    let name = output
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .context("release archive needs a UTF-8 filename")?;
    let preview = format!("parallax-{target}.tar.gz");
    let stable = format!("parallax-{version}-{target}.tar.gz");
    let expected = match channel {
        Channel::Preview => preview,
        Channel::Stable | Channel::Rehearsal => stable,
    };
    if name != expected {
        bail!("release archive name `{name}` does not match {channel:?} identity `{expected}`");
    }
    Ok(())
}

pub(crate) fn rehearse(
    binary: &Path,
    target: &str,
    version: &str,
    channel: Channel,
    source_epoch: u64,
    output_dir: &Path,
) -> Result<()> {
    validate_identity(target, version)?;
    validate_channel_version(version, channel)?;
    validate_binary(binary, target, version)?;
    rehearse_archives(binary, target, version, channel, source_epoch, output_dir)
}

fn rehearse_archives(
    binary: &Path,
    target: &str,
    version: &str,
    channel: Channel,
    source_epoch: u64,
    output_dir: &Path,
) -> Result<()> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("create rehearsal directory {}", output_dir.display()))?;
    let name = match channel {
        Channel::Preview => format!("parallax-{target}.tar.gz"),
        Channel::Stable | Channel::Rehearsal => format!("parallax-{version}-{target}.tar.gz"),
    };
    let first = temporary(output_dir, &name, "first");
    let second = temporary(output_dir, &name, "second");
    archive::write(binary, &first, source_epoch)?;
    archive::write(binary, &second, source_epoch)?;
    let first_digest = archive::digest(&first)?;
    let second_digest = archive::digest(&second)?;
    if first_digest != second_digest {
        bail!(
            "release rehearsal is not deterministic: first={first_digest}, second={second_digest}"
        );
    }

    let final_path = output_dir.join(name);
    std::fs::rename(&first, &final_path)
        .with_context(|| format!("promote rehearsal archive {}", final_path.display()))?;
    std::fs::remove_file(&second)
        .with_context(|| format!("remove rehearsal duplicate {}", second.display()))?;
    archive::write_checksum(&final_path)?;
    println!(
        "==> deterministic rehearsal ready: {} ({first_digest})",
        final_path.display()
    );
    Ok(())
}

fn validate_binary(binary: &Path, target: &str, version: &str) -> Result<()> {
    let size = std::fs::metadata(binary)
        .with_context(|| format!("read release binary metadata {}", binary.display()))?
        .len();
    if size > MAX_RELEASE_BINARY_BYTES {
        bail!("release binary exceeds 512 MiB");
    }
    let binary_bytes = std::fs::read(binary)
        .with_context(|| format!("read release binary {}", binary.display()))?;
    verify::verify_object(&binary_bytes, target, version)
}

pub(crate) fn verify(spec: VerifySpec) -> Result<()> {
    validate_identity(&spec.target, &spec.version)?;
    validate_verification_identity(&spec)?;
    println!("==> verify release set {}", spec.archive.display());
    verify::local(&spec)?;
    verify::signature(&spec)?;
    verify::provenance(&spec)?;
    println!("==> release set verified");
    Ok(())
}

fn validate_verification_identity(spec: &VerifySpec) -> Result<()> {
    if spec.source_commit.len() != 40
        || !spec
            .source_commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || spec
            .source_commit
            .bytes()
            .any(|byte| byte.is_ascii_uppercase())
    {
        bail!("source commit must be one full lowercase SHA");
    }
    let channel = if spec.source_ref == "refs/heads/main" {
        Channel::Preview
    } else if spec.source_ref == format!("refs/tags/v{}", spec.version) {
        Channel::Stable
    } else {
        bail!("source ref must be refs/heads/main for preview or refs/tags/v<version> for stable");
    };
    validate_channel_version(&spec.version, channel)?;
    validate_archive_name(&spec.archive, &spec.target, &spec.version, channel)?;
    let repository_parts = spec.repository.split('/').collect::<Vec<_>>();
    if repository_parts.len() != 2
        || repository_parts.iter().any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        })
    {
        bail!("repository must be one GitHub owner/repository name");
    }
    let workflow_prefix = format!("{}/.github/workflows/", spec.repository);
    if !spec.signer_workflow.starts_with(&workflow_prefix)
        || !spec.signer_workflow.ends_with(".yml")
    {
        bail!("signer workflow must be an exact repository-owned .github workflow");
    }
    let expected_identity = format!(
        "https://github.com/{}@{}",
        spec.signer_workflow, spec.source_ref
    );
    if spec.signer_identity != expected_identity {
        bail!("signer identity must match the repository workflow and source ref");
    }
    Ok(())
}

fn validate_identity(target: &str, version: &str) -> Result<()> {
    if !TARGETS.contains(&target) {
        bail!("unsupported release target `{target}`");
    }
    semver::Version::parse(version)
        .with_context(|| format!("invalid semantic release version `{version}`"))?;
    Ok(())
}

fn validate_channel_version(version: &str, channel: Channel) -> Result<()> {
    let version = semver::Version::parse(version)
        .with_context(|| format!("invalid semantic release version `{version}`"))?;
    match channel {
        Channel::Preview => {
            if !version.pre.as_str().starts_with("preview.") || version.build.is_empty() {
                bail!("preview version must use <version>-preview.<ordinal>+<source> identity");
            }
        }
        Channel::Stable => {
            if !version.pre.is_empty() || !version.build.is_empty() {
                bail!("stable version cannot contain prerelease or build metadata");
            }
        }
        Channel::Rehearsal => {}
    }
    Ok(())
}

fn temporary(directory: &Path, name: &str, pass: &str) -> PathBuf {
    directory.join(format!(".{name}.{pass}.{}", std::process::id()))
}

#[cfg(test)]
mod tests;
