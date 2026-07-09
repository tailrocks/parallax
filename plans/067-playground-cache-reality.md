# Plan 067 Remaining: Playground Cache Reality

## Audit Verdict

Implementation is mostly landed. Remaining blockers are live proof and exact
GreptimeDB metric-name normalization for cache counters.

## Remaining Work

- [ ] Run A26 live and record observed same-SKU hit/miss counts.
- [ ] Query native GreptimeDB metric tables and record the actual cache metric
  names (`cache.hits`/`cache.misses` may normalize to table names such as
  `cache_hits_total`/`cache_misses_total`).
- [ ] Query native logs/spans for `cache.hit`.
- [ ] Record `?stampede=10` trace evidence and compute span count.
- [ ] Update scenario text if the metric names shown to users differ from
  current GreptimeDB table names.

## Remove When

- Live cache metric/log/trace evidence is recorded and scenario text matches
  the backend names.
