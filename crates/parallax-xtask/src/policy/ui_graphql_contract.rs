//! Plan 152 GraphQL contract policy: schema present, probe present, no growth
//! of anonymous transport without legacy handoff ownership via architecture.

use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.graphql-contract";

/// Enforce checked-in GraphQL contract artifacts and generation surface.
#[expect(
    clippy::too_many_lines,
    reason = "one linear contract-artifact checklist"
)]
pub(super) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let schema = root.join("ui/graphql/schema.graphql");
    if schema.is_file() {
        let text =
            fs::read_to_string(&schema).with_context(|| format!("read {}", schema.display()))?;
        if text.trim().is_empty() || !text.contains("type Query") {
            findings.push(Finding::error(
                "ui.graphql-contract.schema.empty",
                "ui/graphql/schema.graphql",
                1,
                "checked-in GraphQL SDL is empty or missing Query",
                "re-export with `cargo xtask ui graphql export`",
                RERUN,
            ));
        }
        if !text.ends_with('\n') || text.contains('\r') {
            findings.push(Finding::error(
                "ui.graphql-contract.schema.normalize",
                "ui/graphql/schema.graphql",
                1,
                "SDL must use LF endings and one trailing newline",
                "re-export with `cargo xtask ui graphql export`",
                RERUN,
            ));
        }
    } else {
        findings.push(Finding::error(
            "ui.graphql-contract.schema.missing",
            "ui/graphql/schema.graphql",
            1,
            "checked-in GraphQL SDL is missing",
            "run `cargo xtask ui graphql export`",
            RERUN,
        ));
    }

    let codegen = root.join("ui/codegen.ts");
    if !codegen.is_file() {
        findings.push(Finding::error(
            "ui.graphql-contract.codegen.missing",
            "ui/codegen.ts",
            1,
            "GraphQL codegen config is missing",
            "restore ui/codegen.ts from Plan 152",
            RERUN,
        ));
    }

    let base = root.join("ui/src/platform/graphql/generated/schema-types.generated.ts");
    if !base.is_file() {
        findings.push(Finding::error(
            "ui.graphql-contract.generated.base",
            "ui/src/platform/graphql/generated/schema-types.generated.ts",
            1,
            "base generated schema types are missing",
            "run `cd ui && bun run graphql:generate`",
            RERUN,
        ));
    }

    let probe_gql = root.join("ui/src/platform/graphql/tests/fixtures/static-probe.graphql");
    let probe_ts = root.join("ui/src/platform/graphql/tests/fixtures/static-probe.generated.ts");
    if !probe_gql.is_file() || !probe_ts.is_file() {
        findings.push(Finding::error(
            "ui.graphql-contract.probe.missing",
            "ui/src/platform/graphql/tests/fixtures",
            1,
            "static GraphQL contract probe document/output missing",
            "restore static-probe.graphql and regenerate",
            RERUN,
        ));
    } else {
        let probe = fs::read_to_string(&probe_gql)?;
        if !probe.contains("query GraphqlContractStaticProbe") {
            findings.push(Finding::error(
                "ui.graphql-contract.probe.name",
                "ui/src/platform/graphql/tests/fixtures/static-probe.graphql",
                1,
                "probe must be the named GraphqlContractStaticProbe operation",
                "restore the Plan 152 probe document",
                RERUN,
            ));
        }
        let generated = fs::read_to_string(&probe_ts)?;
        if !generated.contains("GraphqlContractStaticProbeQuerySchema") {
            findings.push(Finding::error(
                "ui.graphql-contract.probe.schema",
                "ui/src/platform/graphql/tests/fixtures/static-probe.generated.ts",
                1,
                "probe generated output missing Zod operation result schema",
                "run `cd ui && bun run graphql:generate`",
                RERUN,
            ));
        }
    }

    let client = root.join("ui/src/platform/graphql/client.ts");
    if client.is_file() {
        let source = fs::read_to_string(&client)?;
        for needle in [
            "executeGraphqlOperation",
            "executeCachedGraphqlOperation",
            "operationName",
        ] {
            if !source.contains(needle) {
                findings.push(Finding::error(
                    "ui.graphql-contract.client.surface",
                    "ui/src/platform/graphql/client.ts",
                    1,
                    &format!("client missing required surface `{needle}`"),
                    "restore Plan 152 client contract",
                    RERUN,
                ));
            }
        }
    } else {
        findings.push(Finding::error(
            "ui.graphql-contract.client.missing",
            "ui/src/platform/graphql/client.ts",
            1,
            "decoded GraphQL client is missing",
            "restore platform/graphql/client.ts",
            RERUN,
        ));
    }

    let widget = root.join("ui/src/features/dashboards/api/widget-series-operation.ts");
    if !widget.is_file() {
        findings.push(Finding::error(
            "ui.graphql-contract.widget.missing",
            "ui/src/features/dashboards/api/widget-series-operation.ts",
            1,
            "dashboard widget-series AST builder missing",
            "restore features/dashboards/api/widget-series-operation.ts",
            RERUN,
        ));
    }

    Ok(findings)
}
