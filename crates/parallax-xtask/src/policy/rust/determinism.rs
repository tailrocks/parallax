use syn::{Expr, ExprCall};

use crate::diagnostic::Finding;

#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct Metrics {
    pub(super) environment_mutations: usize,
    pub(super) sleeps: usize,
    pub(super) listener_binds: usize,
    pub(super) wall_clocks: usize,
    pub(super) temp_root_accesses: usize,
}

impl Metrics {
    pub(super) fn visit_call(&mut self, call: &ExprCall) {
        let Expr::Path(function) = &*call.func else {
            return;
        };
        let path = function
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if matches!(path.as_str(), "std::env::set_var" | "std::env::remove_var") {
            self.environment_mutations += 1;
        }
        if matches!(path.as_str(), "std::thread::sleep" | "tokio::time::sleep") {
            self.sleeps += 1;
        }
        if path.ends_with("TcpListener::bind") {
            self.listener_binds += 1;
        }
        if matches!(
            path.as_str(),
            "std::time::SystemTime::now"
                | "std::time::Instant::now"
                | "SystemTime::now"
                | "Instant::now"
        ) {
            self.wall_clocks += 1;
        }
        if path == "std::env::temp_dir" {
            self.temp_root_accesses += 1;
        }
    }
}

pub(super) fn findings(relative: &str, metrics: &Metrics) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (rule, value) in [
        (
            "health.rust.test-environment-mutations",
            metrics.environment_mutations,
        ),
        ("health.rust.test-sleeps", metrics.sleeps),
        ("health.rust.test-listener-binds", metrics.listener_binds),
        ("health.rust.test-wall-clocks", metrics.wall_clocks),
        (
            "health.rust.test-temp-root-accesses",
            metrics.temp_root_accesses,
        ),
    ] {
        if value > 0 {
            findings.push(Finding::warning(
                rule,
                relative,
                1,
                &format!("count {value} exceeds target 0"),
                "inject the boundary or use an owned guard/synchronization primitive",
                "cargo xtask health",
            ));
        }
    }
    findings
}
