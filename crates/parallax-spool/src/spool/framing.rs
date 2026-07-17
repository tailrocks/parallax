use super::*;
use std::io::Seek;

impl Spool {
    pub fn line_count(&self, signal: Signal) -> anyhow::Result<usize> {
        let mut total = 0usize;
        let pspl = self.dir.join(signal.file_name());
        if pspl.exists() {
            total = total.saturating_add(count_pspl_frames(&pspl)?);
        }
        let ndjson = self.dir.join(signal.legacy_file_name());
        if ndjson.exists() {
            total = total.saturating_add(std::fs::read_to_string(ndjson)?.lines().count());
        }
        Ok(total)
    }
}

pub(super) fn count_pspl_frames(path: &Path) -> anyhow::Result<usize> {
    let mut file = std::fs::File::open(path)?;
    let mut magic = [0u8; 5];
    let n = file.read(&mut magic)?;
    if n == 0 {
        return Ok(0);
    }
    if n < 5 || &magic != MAGIC {
        return Ok(0);
    }
    let mut count = 0usize;
    loop {
        let mut len_buf = [0u8; 4];
        match file.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        // The length prefix is attacker-controlled on-disk data: seek past
        // the frame instead of allocating up to 4 GiB to skip it. A frame
        // running past EOF counts as truncation, not a frame.
        let len = u32::from_le_bytes(len_buf);
        let position = file.seek(std::io::SeekFrom::Current(i64::from(len)))?;
        if position > file.metadata()?.len() {
            break;
        }
        count += 1;
    }
    Ok(count)
}

pub(super) fn rotated_timestamp(file_name: &str) -> Option<Option<u64>> {
    for signal in Signal::ALL {
        let prefix = format!("{}.", signal.stem());
        for suffix in [".pspl", ".ndjson"] {
            let active = if suffix == ".pspl" {
                signal.file_name()
            } else {
                signal.legacy_file_name()
            };
            if file_name == active
                || !file_name.starts_with(&prefix)
                || !file_name.ends_with(suffix)
            {
                continue;
            }
            let middle = &file_name[prefix.len()..file_name.len() - suffix.len()];
            let timestamp = middle
                .split('-')
                .next()
                .and_then(|part| part.parse::<u64>().ok());
            return Some(timestamp);
        }
    }
    None
}
