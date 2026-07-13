use super::*;
use crate::policy::config::{Architecture, Budgets, Exception, Product};

fn ratchet() -> Ratchet {
    Ratchet {
        schema_version: 1,
        architecture: Architecture { packages: vec![] },
        budgets: Budgets::default(),
        product: Product::default(),
        limits: vec![],
        generated: vec![],
        exceptions: vec![],
        rust_suppressions: vec![],
    }
}

fn exception_ratchet() -> Ratchet {
    Ratchet {
        exceptions: vec![Exception {
            rule: "arch.dependency.direction".into(),
            scope: "low -> high (normal)".into(),
            evidence: "measured".into(),
            owner: "Tailrocks".into(),
            created: "2026-07-12".into(),
            expires: "2026-12-31".into(),
            removal_condition: "plan".into(),
            replacement: "lower tier".into(),
        }],
        ..ratchet()
    }
}
fn nodes() -> BTreeMap<String, Node> {
    [
        (
            "low".into(),
            Node {
                class: "product".into(),
                tier: Some(1),
                agent_context: false,
            },
        ),
        (
            "high".into(),
            Node {
                class: "product".into(),
                tier: Some(4),
                agent_context: false,
            },
        ),
        (
            "aux".into(),
            Node {
                class: "aux".into(),
                tier: None,
                agent_context: false,
            },
        ),
    ]
    .into()
}
fn edge(from: &str, to: &str, kind: Kind) -> Edge {
    Edge {
        from: from.into(),
        to: to.into(),
        kind,
        optional: false,
        target: None,
    }
}

#[test]
fn permits_downward_and_exact_exception() {
    assert!(
        evaluate(
            &nodes(),
            &[edge("low", "high", Kind::Normal)],
            &exception_ratchet()
        )
        .is_empty()
    );
    assert!(evaluate(&nodes(), &[edge("high", "low", Kind::Normal)], &ratchet()).is_empty());
}
#[test]
fn rejects_build_upward_and_aux_edges() {
    let result = evaluate(
        &nodes(),
        &[
            edge("low", "high", Kind::Build),
            edge("high", "aux", Kind::Dev),
        ],
        &ratchet(),
    );
    assert_eq!(result.len(), 2);
}
#[test]
fn detects_production_and_mixed_cycles() {
    let result = evaluate(
        &nodes(),
        &[
            edge("high", "low", Kind::Normal),
            edge("low", "high", Kind::Dev),
        ],
        &ratchet(),
    );
    assert!(result.iter().any(|f| f.rule_id == "arch.mixed-cycle"));
}
#[test]
fn fails_unknown_and_missing_classifications() {
    let mut nodes = nodes();
    nodes.insert(
        "bad".into(),
        Node {
            class: "mystery".into(),
            tier: None,
            agent_context: false,
        },
    );
    let result = evaluate(&nodes, &[edge("missing", "low", Kind::Normal)], &ratchet());
    assert_eq!(result.len(), 2);
}
#[test]
fn includes_optional_target_edges() {
    let mut edge = edge("low", "high", Kind::Build);
    edge.optional = true;
    edge.target = Some("cfg(unix)".into());
    assert_eq!(evaluate(&nodes(), &[edge], &ratchet()).len(), 1);
}

#[test]
fn rejects_stale_and_expired_exceptions() {
    let mut ratchet = exception_ratchet();
    ratchet.exceptions[0].expires = "2020-01-01".into();
    let result = evaluate(&nodes(), &[], &ratchet);
    assert!(
        result
            .iter()
            .any(|finding| finding.rule_id == "arch.exception.expired")
    );
    assert!(
        result
            .iter()
            .any(|finding| finding.rule_id == "arch.exception.stale")
    );
}

#[test]
fn rejects_release_reachability_to_test_support() {
    let mut nodes = nodes();
    nodes.insert(
        "tests".into(),
        Node {
            class: "test-support".into(),
            tier: None,
            agent_context: false,
        },
    );
    let result = evaluate(&nodes, &[edge("high", "tests", Kind::Normal)], &ratchet());
    assert!(
        result
            .iter()
            .any(|finding| finding.rule_id == "arch.test-support-release")
            && evaluate(&nodes, &[edge("high", "tests", Kind::Dev)], &ratchet()).is_empty()
    );
}

#[test]
fn agent_context_rejects_raw_product_dependencies() {
    let mut nodes = nodes();
    nodes.insert(
        "parallax-agent".into(),
        Node {
            class: "product".into(),
            tier: Some(5),
            agent_context: true,
        },
    );
    nodes.insert(
        "parallax-evidence".into(),
        Node {
            class: "product".into(),
            tier: Some(2),
            agent_context: false,
        },
    );
    let allowed = evaluate(
        &nodes,
        &[edge("parallax-agent", "parallax-evidence", Kind::Normal)],
        &ratchet(),
    );
    let rejected = evaluate(
        &nodes,
        &[edge("parallax-agent", "low", Kind::Normal)],
        &ratchet(),
    );
    assert!(allowed.is_empty());
    assert!(
        rejected
            .iter()
            .any(|finding| finding.rule_id == "arch.agent-context.dependency")
    );
}
