use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Context, Result};
use cargo_metadata::{DependencyKind, MetadataCommand};
use time::{Date, OffsetDateTime, macros::format_description};

use crate::diagnostic::Finding;

use super::config::Ratchet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Normal,
    Build,
    Dev,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Build => "build",
            Self::Dev => "dev",
        }
    }
    fn production(self) -> bool {
        self != Self::Dev
    }
}

#[derive(Clone, Debug)]
struct Node {
    class: String,
    tier: Option<u8>,
}

#[derive(Clone, Debug)]
struct Edge {
    from: String,
    to: String,
    kind: Kind,
    optional: bool,
    target: Option<String>,
}

pub fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let metadata = MetadataCommand::new()
        .current_dir(root)
        .no_deps()
        .exec()
        .context("cargo metadata failed")?;
    let members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    let mut edges = Vec::new();
    for package in metadata
        .packages
        .iter()
        .filter(|package| members.contains(&package.id))
    {
        for dependency in &package.dependencies {
            let Some(target_package) = metadata
                .packages
                .iter()
                .find(|candidate| candidate.name == dependency.name)
            else {
                continue;
            };
            if !members.contains(&target_package.id) {
                continue;
            }
            edges.push(Edge {
                from: package.name.to_string(),
                to: target_package.name.to_string(),
                kind: match dependency.kind {
                    DependencyKind::Normal => Kind::Normal,
                    DependencyKind::Build => Kind::Build,
                    DependencyKind::Development => Kind::Dev,
                    _ => Kind::Normal,
                },
                optional: dependency.optional,
                target: dependency.target.as_ref().map(ToString::to_string),
            });
        }
    }
    let nodes = ratchet
        .architecture
        .packages
        .iter()
        .map(|package| {
            (
                package.name.clone(),
                Node {
                    class: package.class.clone(),
                    tier: package.tier,
                },
            )
        })
        .collect();
    let mut findings = evaluate(&nodes, &edges, ratchet);
    let workspace_names: BTreeSet<_> = metadata
        .packages
        .iter()
        .filter(|package| members.contains(&package.id))
        .map(|package| package.name.to_string())
        .collect();
    for name in workspace_names.difference(&nodes.keys().cloned().collect()) {
        findings.push(finding(
            "arch.classification",
            name,
            "workspace package is not classified",
        ));
    }
    for name in nodes.keys().filter(|name| !workspace_names.contains(*name)) {
        findings.push(finding(
            "arch.classification.stale",
            name,
            "classified package is not a workspace member",
        ));
    }
    Ok(findings)
}

fn evaluate(nodes: &BTreeMap<String, Node>, edges: &[Edge], ratchet: &Ratchet) -> Vec<Finding> {
    let mut findings = Vec::new();
    let edge_scopes: BTreeSet<_> = edges
        .iter()
        .map(|edge| format!("{} -> {} ({})", edge.from, edge.to, edge.kind.label()))
        .collect();
    for exception in &ratchet.exceptions {
        let metadata_complete = [
            &exception.evidence,
            &exception.owner,
            &exception.created,
            &exception.expires,
            &exception.removal_condition,
            &exception.replacement,
        ]
        .iter()
        .all(|value| !value.trim().is_empty());
        if exception.rule != "arch.dependency.direction" || !metadata_complete {
            findings.push(finding(
                "arch.exception.invalid",
                &exception.scope,
                "exception has an unknown rule or incomplete common metadata",
            ));
            continue;
        }
        let format = format_description!("[year]-[month]-[day]");
        let created = Date::parse(&exception.created, format);
        let expires = Date::parse(&exception.expires, format);
        if created.is_err() || expires.is_err() {
            findings.push(finding(
                "arch.exception.invalid",
                &exception.scope,
                "exception dates must use valid YYYY-MM-DD values",
            ));
            continue;
        }
        if expires.expect("checked above") < OffsetDateTime::now_utc().date() {
            findings.push(finding(
                "arch.exception.expired",
                &exception.scope,
                "exception expiry is in the past",
            ));
        }
        if !edge_scopes.contains(&exception.scope) {
            findings.push(finding(
                "arch.exception.stale",
                &exception.scope,
                "exception no longer matches a workspace edge",
            ));
        }
    }
    for (name, node) in nodes {
        if node.class == "product" && node.tier.is_none() {
            findings.push(finding(
                "arch.classification",
                name,
                "product package has no tier",
            ));
        }
        if !matches!(
            node.class.as_str(),
            "product" | "aux" | "proof" | "test-support"
        ) {
            findings.push(finding(
                "arch.classification",
                name,
                "package has an unknown class",
            ));
        }
    }
    for edge in edges {
        let Some(from) = nodes.get(&edge.from) else {
            findings.push(finding(
                "arch.classification",
                &edge.from,
                "workspace package is not classified",
            ));
            continue;
        };
        let Some(to) = nodes.get(&edge.to) else {
            findings.push(finding(
                "arch.classification",
                &edge.to,
                "workspace dependency is not classified",
            ));
            continue;
        };
        if from.class == "product" && matches!(to.class.as_str(), "aux" | "proof") {
            findings.push(finding(
                "arch.aux-dependency",
                &edge.from,
                &format!(
                    "product package depends on {} package {}",
                    to.class, edge.to
                ),
            ));
        }
        if edge.kind.production()
            && from.class == "product"
            && to.class == "product"
            && from.tier <= to.tier
        {
            let scope = format!("{} -> {} ({})", edge.from, edge.to, edge.kind.label());
            if !ratchet.exceptions.iter().any(|exception| {
                exception.rule == "arch.dependency.direction" && exception.scope == scope
            }) {
                findings.push(finding(
                    "arch.dependency.direction",
                    &edge.from,
                    &format!("forbidden dependency {scope}"),
                ));
            }
        }
        let _edge_shape = (edge.optional, edge.target.as_deref());
    }
    findings.extend(cycles(nodes, edges, true));
    findings.extend(cycles(nodes, edges, false));
    findings
}

fn cycles(nodes: &BTreeMap<String, Node>, edges: &[Edge], production_only: bool) -> Vec<Finding> {
    fn visit(
        name: &str,
        nodes: &BTreeMap<String, Node>,
        edges: &[Edge],
        production_only: bool,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> Option<String> {
        if active.contains(name) {
            return Some(name.to_owned());
        }
        if !done.insert(name.to_owned()) {
            return None;
        }
        active.insert(name.to_owned());
        for edge in edges
            .iter()
            .filter(|edge| edge.from == name && (!production_only || edge.kind.production()))
        {
            if nodes.contains_key(&edge.to)
                && let Some(cycle) = visit(&edge.to, nodes, edges, production_only, active, done)
            {
                return Some(cycle);
            }
        }
        active.remove(name);
        None
    }
    let mut done = BTreeSet::new();
    for name in nodes.keys() {
        if let Some(cycle) = visit(
            name,
            nodes,
            edges,
            production_only,
            &mut BTreeSet::new(),
            &mut done,
        ) {
            return vec![finding(
                if production_only {
                    "arch.production-cycle"
                } else {
                    "arch.mixed-cycle"
                },
                &cycle,
                "workspace dependency cycle detected",
            )];
        }
    }
    Vec::new()
}

fn finding(rule: &str, package: &str, reason: &str) -> Finding {
    Finding::error(
        rule,
        "ratchet.toml",
        1,
        reason,
        &format!("correct classification or dependency for {package}"),
        "cargo xtask arch",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::config::{Architecture, Budgets, Exception, Product};

    fn ratchet() -> Ratchet {
        Ratchet {
            schema_version: 1,
            architecture: Architecture { packages: vec![] },
            budgets: Budgets::default(),
            product: Product::default(),
            limits: vec![],
            exceptions: vec![],
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
                },
            ),
            (
                "high".into(),
                Node {
                    class: "product".into(),
                    tier: Some(3),
                },
            ),
            (
                "aux".into(),
                Node {
                    class: "aux".into(),
                    tier: None,
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
}
