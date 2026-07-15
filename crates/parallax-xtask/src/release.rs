use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

mod archive;
mod verify;

const TARGETS: [&str; 4] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
];

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
    source_epoch: u64,
) -> Result<()> {
    validate_identity(target, version)?;
    validate_archive_name(output, target, version)?;
    let binary_bytes = std::fs::read(binary)
        .with_context(|| format!("read release binary {}", binary.display()))?;
    verify::verify_object(&binary_bytes, target, version)?;
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

fn validate_archive_name(output: &Path, target: &str, version: &str) -> Result<()> {
    let name = output
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .context("release archive needs a UTF-8 filename")?;
    let preview = format!("parallax-{target}.tar.gz");
    let stable = format!("parallax-{version}-{target}.tar.gz");
    if name != preview && name != stable {
        bail!("release archive name `{name}` does not match preview or stable identity");
    }
    Ok(())
}

pub(crate) fn rehearse(
    binary: &Path,
    target: &str,
    version: &str,
    source_epoch: u64,
    output_dir: &Path,
) -> Result<()> {
    validate_identity(target, version)?;
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("create rehearsal directory {}", output_dir.display()))?;
    let name = format!("parallax-{version}-{target}.tar.gz");
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
    if !spec.source_ref.starts_with("refs/") || spec.source_ref.contains(char::is_whitespace) {
        bail!("source ref must be a full refs/* name");
    }
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

fn temporary(directory: &Path, name: &str, pass: &str) -> PathBuf {
    directory.join(format!(".{name}.{pass}.{}", std::process::id()))
}

#[cfg(test)]
mod tests;
