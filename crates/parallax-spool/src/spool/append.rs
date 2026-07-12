use super::*;

impl Spool {
    pub async fn append_raw(&self, signal: Signal, raw: &bytes::Bytes) -> anyhow::Result<()> {
        let payload = raw.to_vec();
        let write_len = u64::try_from(payload.len().saturating_add(4)).unwrap_or(u64::MAX);
        let dir = self.dir.clone();
        let max_segment_bytes = self.max_segment_bytes;

        let mut state = self.states[signal.index()].lock().await;
        let size = state.size;
        let file = state.file.take();

        let (next_file, next_size) = tokio::task::spawn_blocking(move || {
            append_blocking(
                dir,
                signal,
                max_segment_bytes,
                size,
                write_len,
                file,
                payload,
            )
        })
        .await
        .map_err(|e| anyhow::anyhow!("spool append join: {e}"))??;

        state.file = next_file;
        state.size = next_size;
        Ok(())
    }
}

fn append_blocking(
    dir: PathBuf,
    signal: Signal,
    max_segment_bytes: u64,
    mut size: u64,
    write_len: u64,
    mut file: Option<std::fs::File>,
    payload: Vec<u8>,
) -> anyhow::Result<(Option<std::fs::File>, u64)> {
    let needs_magic = size == 0;
    let total_write = if needs_magic {
        write_len.saturating_add(MAGIC.len() as u64)
    } else {
        write_len
    };

    if size > 0 && size.saturating_add(total_write) > max_segment_bytes {
        drop(file.take());
        rotate_active(&dir, signal)?;
        size = 0;
    }

    let needs_magic = size == 0;
    let total_write = if needs_magic {
        write_len.saturating_add(MAGIC.len() as u64)
    } else {
        write_len
    };

    let mut handle = match file {
        Some(handle) => handle,
        None => std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(signal.file_name()))?,
    };
    if needs_magic {
        handle.write_all(MAGIC)?;
    }
    let len = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    handle.write_all(&len.to_le_bytes())?;
    handle.write_all(&payload)?;
    Ok((Some(handle), size.saturating_add(total_write)))
}

fn rotate_active(dir: &Path, signal: Signal) -> anyhow::Result<()> {
    let active = dir.join(signal.file_name());
    if !active.exists() {
        return Ok(());
    }
    let rotated = next_rotated_path(dir, signal);
    std::fs::rename(active, rotated)?;
    Ok(())
}

fn next_rotated_path(dir: &Path, signal: Signal) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let first = dir.join(format!("{}.{}.pspl", signal.stem(), timestamp));
    if !first.exists() {
        return first;
    }
    for sequence in 1u64.. {
        let candidate = dir.join(format!("{}.{}-{}.pspl", signal.stem(), timestamp, sequence));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("unbounded sequence finds a rotated spool path")
}
