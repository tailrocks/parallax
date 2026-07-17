//! Drain-style log pattern mining (plan 165).
//!
//! Query-time only: masks volatile tokens, tokenizes bodies, and clusters them
//! into fixed-depth templates with a similarity threshold and an LRU cluster
//! cap. Pure and deterministic given a fixed input order; cluster ranking is
//! by count descending with stable template-string tie-break.
//!
//! Preliminary slice — peer should verify masking quality, wire GraphQL
//! `logPatterns`, and deepen performance bounds on live 10k-body samples.

use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::sync::OnceLock;

/// Placeholder substituted for masked volatile tokens.
pub const WILDCARD: &str = "<*>";

/// Default Drain tree depth (token positions used as tree path).
pub const DEFAULT_DEPTH: usize = 4;
/// Default token-similarity threshold for joining an existing cluster.
pub const DEFAULT_SIMILARITY: f64 = 0.4;
/// Default maximum number of live clusters (LRU eviction beyond this).
pub const DEFAULT_MAX_CLUSTERS: usize = 512;

/// Configuration for a clustering pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrainConfig {
    pub depth: usize,
    pub similarity_threshold: f64,
    pub max_clusters: usize,
}

impl Default for DrainConfig {
    fn default() -> Self {
        Self {
            depth: DEFAULT_DEPTH,
            similarity_threshold: DEFAULT_SIMILARITY,
            max_clusters: DEFAULT_MAX_CLUSTERS,
        }
    }
}

/// One input log line for clustering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLineInput<'a> {
    pub body: &'a str,
    pub severity: Option<&'a str>,
    pub timestamp_nanos: Option<u64>,
    pub log_id: Option<&'a str>,
}

/// One extracted pattern cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogPatternCluster {
    /// Template with `<*>` wildcards for variable tokens.
    pub template: String,
    pub count: u64,
    /// Severity histogram keyed by the raw severity string (or `"unknown"`).
    pub severity_counts: Vec<(String, u64)>,
    pub first_nanos: Option<u64>,
    pub last_nanos: Option<u64>,
    pub sample_log_id: Option<String>,
}

struct Maskers {
    uuid: Regex,
    ipv4: Regex,
    ipv6: Regex,
    email: Regex,
    hex: Regex,
    number: Regex,
    whitespace: Regex,
}

#[expect(clippy::expect_used, reason = "static regex literal")]
fn static_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex")
}

fn maskers() -> &'static Maskers {
    static CELL: OnceLock<Maskers> = OnceLock::new();
    CELL.get_or_init(|| Maskers {
        // Standard UUID form.
        uuid: static_regex(r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b"),
        ipv4: static_regex(
            r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b",
        ),
        // Compressed IPv6 is hard; cover common full/abbreviated forms loosely.
        ipv6: static_regex(r"(?i)\b(?:[0-9a-f]{1,4}:){2,7}[0-9a-f]{1,4}\b"),
        email: static_regex(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"),
        // Hex ids of length ≥ 8 (trace/span-ish and opaque tokens).
        hex: static_regex(r"(?i)\b[0-9a-f]{8,}\b"),
        // Digit runs (incl. glued units like "42ms" → "<*>ms"). No word-boundary
        // requirement: unit suffixes are common in log bodies.
        number: static_regex(r"\d+(?:\.\d+)?"),
        whitespace: static_regex(r"\s+"),
    })
}

/// Mask volatile token classes in a log body (UUID / IP / hex≥8 / email / number).
#[must_use]
pub fn mask_body(body: &str) -> String {
    let m = maskers();
    let mut out = body.to_string();
    out = m.uuid.replace_all(&out, WILDCARD).into_owned();
    out = m.email.replace_all(&out, WILDCARD).into_owned();
    out = m.ipv4.replace_all(&out, WILDCARD).into_owned();
    out = m.ipv6.replace_all(&out, WILDCARD).into_owned();
    out = m.hex.replace_all(&out, WILDCARD).into_owned();
    out = m.number.replace_all(&out, WILDCARD).into_owned();
    out = m.whitespace.replace_all(&out, " ").into_owned();
    out.trim().to_string()
}

/// Tokenize a (preferably already-masked) body on whitespace.
#[must_use]
pub fn tokenize(body: &str) -> Vec<String> {
    body.split_whitespace()
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// Token-level similarity = matching positions / max length (Drain-style).
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "token counts for similarity are small (log line lengths)"
)]
pub fn token_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0;
    }
    let min_len = a.len().min(b.len());
    let mut matches = 0usize;
    for i in 0..min_len {
        if a[i] == b[i] || a[i] == WILDCARD || b[i] == WILDCARD {
            matches += 1;
        }
    }
    matches as f64 / max_len as f64
}

/// Merge two equal-length-preferred token sequences into a template (position-wise
/// wildcard when tokens disagree). Result length is the longer sequence; extra
/// tokens on the longer side become wildcards only if the shorter side is empty
/// at that position — otherwise they stay as the longer sequence's tokens so a
/// later similarity pass can still split.
#[must_use]
pub fn merge_template(a: &[String], b: &[String]) -> Vec<String> {
    let len = a.len().max(b.len());
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        match (a.get(i), b.get(i)) {
            (Some(x), Some(y)) if x == y => out.push(x.clone()),
            (Some(x), Some(_)) if x == WILDCARD => out.push(WILDCARD.to_string()),
            (Some(_), Some(y)) if y == WILDCARD => out.push(WILDCARD.to_string()),
            (Some(_), Some(_)) => out.push(WILDCARD.to_string()),
            (Some(x), None) | (None, Some(x)) => out.push(x.clone()),
            (None, None) => {}
        }
    }
    out
}

fn template_string(tokens: &[String]) -> String {
    tokens.join(" ")
}

/// Path key for the fixed-depth prefix tree: first token length + first
/// `depth-1` tokens (or `<*>` when short). Groups candidates before the
/// similarity scan, matching Drain's length-then-prefix routing.
fn tree_path(tokens: &[String], depth: usize) -> String {
    let d = depth.max(1);
    let mut parts = Vec::with_capacity(d);
    parts.push(format!("len:{}", tokens.len()));
    for i in 0..d.saturating_sub(1) {
        parts.push(
            tokens
                .get(i)
                .cloned()
                .unwrap_or_else(|| WILDCARD.to_string()),
        );
    }
    parts.join("/")
}

#[derive(Debug, Clone)]
struct Cluster {
    tokens: Vec<String>,
    count: u64,
    severity_counts: HashMap<String, u64>,
    first_nanos: Option<u64>,
    last_nanos: Option<u64>,
    sample_log_id: Option<String>,
}

/// Cluster log bodies with a Drain-style fixed-depth tree + similarity merge.
///
/// Input order affects which sample_log_id is retained (first wins) and the
/// LRU touch order under the cluster cap; ranking of the returned list is
/// stable: count descending, then template ascending.
#[must_use]
pub fn cluster_logs(lines: &[LogLineInput<'_>], config: DrainConfig) -> Vec<LogPatternCluster> {
    let depth = config.depth.max(1);
    let threshold = config.similarity_threshold.clamp(0.0, 1.0);
    let max_clusters = config.max_clusters.max(1);

    // path -> ordered cluster ids in that leaf (for local similarity search).
    let mut leaves: HashMap<String, Vec<usize>> = HashMap::new();
    let mut clusters: Vec<Cluster> = Vec::new();
    // LRU of cluster indices (back = most recent).
    let mut lru: VecDeque<usize> = VecDeque::new();

    for line in lines {
        let masked = mask_body(line.body);
        let tokens = tokenize(&masked);
        if tokens.is_empty() {
            continue;
        }
        let path = tree_path(&tokens, depth);

        let best = find_best_cluster(&tokens, leaves.get(&path), &clusters, threshold);

        if let Some((idx, _)) = best {
            let cluster = &mut clusters[idx];
            cluster.tokens = merge_template(&cluster.tokens, &tokens);
            cluster.count = cluster.count.saturating_add(1);
            let sev = line.severity.unwrap_or("unknown").to_string();
            *cluster.severity_counts.entry(sev).or_insert(0) += 1;
            if let Some(ts) = line.timestamp_nanos {
                cluster.first_nanos = Some(cluster.first_nanos.map_or(ts, |f| f.min(ts)));
                cluster.last_nanos = Some(cluster.last_nanos.map_or(ts, |l| l.max(ts)));
            }
            // Keep the first sample id (stable under re-process of the same window).
            if cluster.sample_log_id.is_none() {
                cluster.sample_log_id = line.log_id.map(str::to_string);
            }
            touch_lru(&mut lru, idx);
        } else {
            if live_cluster_count(&clusters) >= max_clusters {
                evict_to_cap(&mut clusters, &mut leaves, &mut lru, depth, max_clusters);
            }

            let mut severity_counts = HashMap::new();
            let sev = line.severity.unwrap_or("unknown").to_string();
            severity_counts.insert(sev, 1);
            let idx = clusters.len();
            clusters.push(Cluster {
                tokens,
                count: 1,
                severity_counts,
                first_nanos: line.timestamp_nanos,
                last_nanos: line.timestamp_nanos,
                sample_log_id: line.log_id.map(str::to_string),
            });
            leaves.entry(path).or_default().push(idx);
            lru.push_back(idx);
        }
    }

    let mut out: Vec<LogPatternCluster> = clusters
        .into_iter()
        .filter(|c| c.count > 0 && !c.tokens.is_empty())
        .map(|c| {
            let mut severity_counts: Vec<(String, u64)> = c.severity_counts.into_iter().collect();
            severity_counts.sort_by(|a, b| a.0.cmp(&b.0));
            LogPatternCluster {
                template: template_string(&c.tokens),
                count: c.count,
                severity_counts,
                first_nanos: c.first_nanos,
                last_nanos: c.last_nanos,
                sample_log_id: c.sample_log_id,
            }
        })
        .collect();

    out.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.template.cmp(&b.template))
    });
    out
}

fn touch_lru(lru: &mut VecDeque<usize>, idx: usize) {
    if let Some(pos) = lru.iter().position(|&i| i == idx) {
        lru.remove(pos);
    }
    lru.push_back(idx);
}

fn live_cluster_count(clusters: &[Cluster]) -> usize {
    clusters.iter().filter(|c| c.count > 0).count()
}

fn find_best_cluster(
    tokens: &[String],
    leaf: Option<&Vec<usize>>,
    clusters: &[Cluster],
    threshold: f64,
) -> Option<(usize, f64)> {
    let leaf = leaf?;
    let mut best: Option<(usize, f64)> = None;
    for &idx in leaf {
        let sim = token_similarity(tokens, &clusters[idx].tokens);
        if sim < threshold {
            continue;
        }
        match best {
            Some((_, best_sim)) if sim <= best_sim => {}
            _ => best = Some((idx, sim)),
        }
    }
    best
}

/// Evict least-recently-used live clusters until under `max_clusters`.
fn evict_to_cap(
    clusters: &mut [Cluster],
    leaves: &mut HashMap<String, Vec<usize>>,
    lru: &mut VecDeque<usize>,
    depth: usize,
    max_clusters: usize,
) {
    while live_cluster_count(clusters) >= max_clusters {
        let Some(victim) = lru.pop_front() else {
            break;
        };
        let Some(c) = clusters.get_mut(victim) else {
            continue;
        };
        if c.count == 0 {
            continue;
        }
        let victim_path = tree_path(&c.tokens, depth);
        if let Some(ids) = leaves.get_mut(&victim_path) {
            ids.retain(|&i| i != victim);
        }
        c.tokens.clear();
        c.count = 0;
    }
}

#[cfg(test)]
mod tests;
