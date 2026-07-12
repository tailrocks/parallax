use super::*;

impl Spool {
    pub fn reap(&self, retention: SpoolRetention, now: SystemTime) -> anyhow::Result<SpoolReclaim> {
        let mut reclaim = SpoolReclaim::default();
        let now_secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let max_age_secs = retention.max_age.as_secs();
        let mut rotated = self.rotated_segments()?;
        let mut kept = Vec::new();

        for segment in rotated.drain(..) {
            let expired = segment
                .timestamp_secs
                .is_some_and(|timestamp| now_secs.saturating_sub(timestamp) > max_age_secs);
            if expired {
                reclaim.add_removed(&segment)?;
            } else {
                kept.push(segment);
            }
        }

        let rotated_total = kept.iter().map(|segment| segment.size).sum::<u64>();
        let mut total = self.active_total_bytes()?.saturating_add(rotated_total);
        if total > retention.max_total_bytes {
            kept.sort_by_key(|segment| segment.timestamp_secs.unwrap_or(u64::MAX));
            for segment in kept {
                if total <= retention.max_total_bytes {
                    break;
                }
                reclaim.add_removed(&segment)?;
                total = total.saturating_sub(segment.size);
            }
        }

        Ok(reclaim)
    }

    fn active_total_bytes(&self) -> anyhow::Result<u64> {
        let mut total = 0u64;
        for signal in Signal::ALL {
            for name in [signal.file_name(), signal.legacy_file_name()] {
                let path = self.dir.join(name);
                if let Ok(metadata) = path.metadata() {
                    total = total.saturating_add(metadata.len());
                }
            }
        }
        Ok(total)
    }

    fn rotated_segments(&self) -> anyhow::Result<Vec<RotatedSegment>> {
        let mut segments = Vec::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if let Some(timestamp_secs) = rotated_timestamp(file_name) {
                segments.push(RotatedSegment {
                    size: entry.metadata()?.len(),
                    path,
                    timestamp_secs,
                });
            }
        }
        Ok(segments)
    }
}

impl SpoolReclaim {
    fn add_removed(&mut self, segment: &RotatedSegment) -> anyhow::Result<()> {
        std::fs::remove_file(&segment.path)?;
        self.removed_segments += 1;
        self.reclaimed_bytes = self.reclaimed_bytes.saturating_add(segment.size);
        Ok(())
    }
}
