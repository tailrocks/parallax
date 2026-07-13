use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

mod archive;

const TARGETS: [&str; 4] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
];

pub(crate) fn package(binary: &Path, output: &Path, source_epoch: u64) -> Result<()> {
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

fn validate_identity(target: &str, version: &str) -> Result<()> {
    if !TARGETS.contains(&target) {
        bail!("unsupported release target `{target}`");
    }
    if version.is_empty()
        || version.starts_with('v')
        || version.contains('/')
        || version.contains(char::is_whitespace)
    {
        bail!("invalid release version `{version}`");
    }
    Ok(())
}

fn temporary(directory: &Path, name: &str, pass: &str) -> PathBuf {
    directory.join(format!(".{name}.{pass}.{}", std::process::id()))
}

#[cfg(test)]
mod tests;
