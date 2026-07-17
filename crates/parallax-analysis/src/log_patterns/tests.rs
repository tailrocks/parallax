use super::*;

#[test]
fn masks_uuid_ip_email_hex_and_numbers() {
    let body = "user 550e8400-e29b-41d4-a716-446655440000 from 10.0.0.7 \
        mailed a@b.co token deadbeefcafebabe waited 42ms";
    let masked = mask_body(body);
    assert!(
        !masked.contains("550e8400"),
        "uuid must not survive: {masked}"
    );
    assert!(
        !masked.contains("10.0.0.7"),
        "ipv4 must not survive: {masked}"
    );
    assert!(
        !masked.contains("a@b.co"),
        "email must not survive: {masked}"
    );
    assert!(
        !masked.contains("deadbeefcafebabe"),
        "hex≥8 must not survive: {masked}"
    );
    assert!(!masked.contains("42"), "numbers must not survive: {masked}");
    assert_eq!(
        masked.matches(WILDCARD).count(),
        5,
        "expected five wildcards, got: {masked}"
    );
}

#[test]
fn template_stable_across_parameter_churn() {
    let lines = [
        LogLineInput {
            body: "checkout authorize user=u-111 duration=12ms",
            severity: Some("INFO"),
            timestamp_nanos: Some(100),
            log_id: Some("a"),
        },
        LogLineInput {
            body: "checkout authorize user=u-222 duration=99ms",
            severity: Some("INFO"),
            timestamp_nanos: Some(200),
            log_id: Some("b"),
        },
        LogLineInput {
            body: "checkout authorize user=u-333 duration=5ms",
            severity: Some("WARN"),
            timestamp_nanos: Some(300),
            log_id: Some("c"),
        },
    ];
    let clusters = cluster_logs(&lines, DrainConfig::default());
    assert_eq!(
        clusters.len(),
        1,
        "parameter churn must one template: {clusters:?}"
    );
    assert_eq!(clusters[0].count, 3);
    assert!(
        clusters[0].template.contains(WILDCARD),
        "template should mask params: {}",
        clusters[0].template
    );
    assert!(
        clusters[0].template.contains("checkout") && clusters[0].template.contains("authorize"),
        "stable tokens remain: {}",
        clusters[0].template
    );
    assert_eq!(clusters[0].sample_log_id.as_deref(), Some("a"));
    assert_eq!(clusters[0].first_nanos, Some(100));
    assert_eq!(clusters[0].last_nanos, Some(300));
    // severity mix captured
    let info = clusters[0]
        .severity_counts
        .iter()
        .find(|(k, _)| k == "INFO")
        .map(|(_, n)| *n);
    let warn = clusters[0]
        .severity_counts
        .iter()
        .find(|(k, _)| k == "WARN")
        .map(|(_, n)| *n);
    assert_eq!(info, Some(2));
    assert_eq!(warn, Some(1));
}

#[test]
fn distinct_templates_do_not_merge_below_threshold() {
    let lines = [
        LogLineInput {
            body: "payment charged card visa amount 10",
            severity: Some("INFO"),
            timestamp_nanos: None,
            log_id: Some("1"),
        },
        LogLineInput {
            body: "inventory reserved sku widget qty 3",
            severity: Some("INFO"),
            timestamp_nanos: None,
            log_id: Some("2"),
        },
    ];
    let clusters = cluster_logs(
        &lines,
        DrainConfig {
            similarity_threshold: 0.4,
            ..DrainConfig::default()
        },
    );
    assert_eq!(
        clusters.len(),
        2,
        "unrelated messages must not merge: {clusters:?}"
    );
}

#[test]
fn spiking_template_ranks_first_by_count() {
    let mut lines = Vec::new();
    for i in 0..20 {
        lines.push(format!(
            "spike path handler latency={i}ms correlation={i:08x}"
        ));
    }
    for i in 0..3 {
        lines.push(format!("quiet background tick n={i}"));
    }
    let inputs: Vec<LogLineInput<'_>> = lines
        .iter()
        .enumerate()
        .map(|(i, body)| LogLineInput {
            body,
            severity: Some("ERROR"),
            timestamp_nanos: Some(i as u64),
            log_id: None,
        })
        .collect();
    let clusters = cluster_logs(&inputs, DrainConfig::default());
    assert!(clusters.len() >= 2, "expected ≥2 clusters: {clusters:?}");
    assert_eq!(clusters[0].count, 20, "spike must rank first: {clusters:?}");
    assert!(
        clusters[0].template.contains("spike"),
        "top template is the spike: {}",
        clusters[0].template
    );
}

#[test]
fn token_similarity_handles_wildcards_and_empty() {
    let a = tokenize(&mask_body("a b c"));
    let b = tokenize(&mask_body("a x c"));
    let sim = token_similarity(&a, &b);
    assert!((sim - 2.0 / 3.0).abs() < 1e-9, "sim={sim}");
    assert!((token_similarity(&[], &[]) - 1.0).abs() < 1e-9);
}

#[test]
fn lru_cap_bounds_cluster_count() {
    let mut lines = Vec::new();
    for i in 0..30 {
        // Fully distinct templates so each creates a new cluster.
        lines.push(format!("unique word{i} alpha{i} beta{i} gamma{i}"));
    }
    let inputs: Vec<LogLineInput<'_>> = lines
        .iter()
        .map(|body| LogLineInput {
            body,
            severity: None,
            timestamp_nanos: None,
            log_id: None,
        })
        .collect();
    let clusters = cluster_logs(
        &inputs,
        DrainConfig {
            max_clusters: 10,
            similarity_threshold: 1.0, // no merges
            ..DrainConfig::default()
        },
    );
    assert!(
        clusters.len() <= 10,
        "LRU cap must bound live clusters: got {}",
        clusters.len()
    );
}

#[test]
fn ten_thousand_lines_complete_quickly() {
    // Distinct *stable* template tokens (not numeric — numbers get masked).
    const NAMES: [&str; 12] = [
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima",
    ];
    let mut bodies = Vec::with_capacity(10_000);
    for i in 0..10_000 {
        let name = NAMES[i % 12];
        bodies.push(format!(
            "service-handler-{name} request id={i:08x} from 10.0.0.1 took {i}ms"
        ));
    }
    let inputs: Vec<LogLineInput<'_>> = bodies
        .iter()
        .map(|body| LogLineInput {
            body,
            severity: Some("INFO"),
            timestamp_nanos: None,
            log_id: None,
        })
        .collect();
    let start = std::time::Instant::now();
    let clusters = cluster_logs(&inputs, DrainConfig::default());
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 2_000,
        "10k lines must finish under 2s, took {elapsed:?}"
    );
    // ~12 templates with parameter churn; allow some over-split from depth routing.
    assert!(
        (8..=24).contains(&clusters.len()),
        "expected ~12 clusters, got {}: {:?}",
        clusters.len(),
        clusters
            .iter()
            .map(|c| (&c.template, c.count))
            .collect::<Vec<_>>()
    );
    let total: u64 = clusters.iter().map(|c| c.count).sum();
    assert_eq!(total, 10_000);
}
