use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use flate2::{Compression, GzBuilder};
use sha2::{Digest, Sha256};
use tar::{Builder, EntryType, Header};

const BUFFER_BYTES: usize = 64 * 1024;

pub(super) fn write(binary: &Path, output: &Path, source_epoch: u64) -> Result<()> {
    let mut input =
        File::open(binary).with_context(|| format!("open release binary {}", binary.display()))?;
    let size = input
        .metadata()
        .with_context(|| format!("read release binary metadata {}", binary.display()))?
        .len();
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create archive directory {}", parent.display()))?;
    }
    let destination = File::create(output)
        .with_context(|| format!("create release archive {}", output.display()))?;
    let encoder = GzBuilder::new()
        .mtime(0)
        .operating_system(255)
        .write(destination, Compression::best());
    let mut builder = Builder::new(encoder);
    let mut header = Header::new_gnu();
    header.set_size(size);
    header.set_mode(0o755);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(source_epoch);
    header.set_username("root")?;
    header.set_groupname("root")?;
    header.set_entry_type(EntryType::Regular);
    header.set_cksum();
    builder.append_data(&mut header, "parallax", &mut input)?;
    let encoder = builder.into_inner()?;
    let output = encoder.finish()?;
    output.sync_all()?;
    Ok(())
}

pub(super) fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut hasher = Sha256::new();
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn write_checksum(archive: &Path) -> Result<String> {
    let digest = digest(archive)?;
    let checksum = archive.with_extension(format!(
        "{}.sha256",
        archive
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
    ));
    let mut file = File::create(&checksum)
        .with_context(|| format!("create checksum {}", checksum.display()))?;
    writeln!(file, "{digest}")?;
    file.sync_all()?;
    Ok(digest)
}
