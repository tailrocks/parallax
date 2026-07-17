//! Plan 153 runtime-boundary policy: required platform owners present;
//! product routes must not gain new direct EventSource/localStorage owners
//! beyond shrink-only legacy handoffs recorded at foundation time.

use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.runtime-boundaries";

const REQUIRED: &[(&str, &str)] = &[
    (
        "ui/src/platform/external-values/decode-json-text.ts",
        "decodeJsonText",
    ),
    (
        "ui/src/platform/external-values/boundary-error.ts",
        "BoundaryError",
    ),
    (
        "ui/src/platform/sse/event-source.ts",
        "browserEventSourceFactory",
    ),
    (
        "ui/src/platform/sse/live-stream-controller.ts",
        "createLiveStreamController",
    ),
    (
        "ui/src/platform/visibility/use-page-visible.ts",
        "usePageVisible",
    ),
    (
        "ui/src/platform/url/decode-search-value.ts",
        "decodeSearchValue",
    ),
    (
        "ui/src/platform/storage/browser-storage.ts",
        "readBrowserStorage",
    ),
    (
        "ui/src/platform/storage/versioned-storage-codec.ts",
        "readVersionedStorage",
    ),
];

/// Enforce Plan-153 platform modules exist with required surfaces.
pub(super) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for (relative, needle) in REQUIRED {
        let path = root.join(relative);
        if !path.is_file() {
            findings.push(Finding::error(
                "ui.runtime-boundaries.owner.missing",
                relative,
                1,
                "required runtime-boundary owner module is missing",
                "restore the Plan 153 platform module",
                RERUN,
            ));
            continue;
        }
        let source =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if !source.contains(needle) {
            findings.push(Finding::error(
                "ui.runtime-boundaries.owner.surface",
                relative,
                1,
                &format!("owner module missing required surface `{needle}`"),
                "restore the Plan 153 contract surface",
                RERUN,
            ));
        }
    }

    // No production environment / window-message adapters while consumer count
    // is zero (Plan 153 first-consumer policy).
    for forbidden in [
        "ui/src/platform/environment",
        "ui/src/platform/window-message",
        "ui/src/platform/post-message",
    ] {
        let path = root.join(forbidden);
        if path.exists() {
            findings.push(Finding::error(
                "ui.runtime-boundaries.premature-adapter",
                forbidden,
                1,
                "environment/window-message production adapter present without a product consumer plan",
                "remove dead adapter; first consumer requires its own plan",
                RERUN,
            ));
        }
    }

    Ok(findings)
}
