//! Workspace-wide TypeScript boundary and health orchestration.

use super::*;

pub(in crate::policy) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let ui = root.join("ui");
    let provider = TypeScriptProvider::new(&ui.join("tsconfig.json"));
    let mut files = Vec::new();
    collect_source_files(&ui.join("src"), &mut files)?;
    if ui.join("tests").is_dir() {
        collect_source_files(&ui.join("tests"), &mut files)?;
    }
    for config in ["vite.config.ts", "eslint.config.js", "prettier.config.js"] {
        let path = ui.join(config);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    let mut findings = Vec::new();
    let mut graph = BTreeMap::<PathBuf, Vec<PathBuf>>::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let analysis = provider.analyze(&path, &source);
        findings.extend(analysis.findings);
        if analysis.star_exports > 0
            && path.file_name().and_then(|name| name.to_str()) != Some("routeTree.gen.ts")
        {
            findings.push(finding(
                &path,
                1,
                "typescript.star-export",
                "handwritten export-star barrels hide public-surface growth",
            ));
        }
        for edge in analysis.imports {
            if !edge.resolved.starts_with(&ui) {
                continue;
            }
            check_boundary(&ui, &path, &edge, &mut findings);
            graph.entry(path.clone()).or_default().push(edge.resolved);
        }
    }
    if let Some(path) = find_cycle(&graph) {
        findings.push(finding(
            &path,
            1,
            "typescript.import-cycle",
            "TypeScript module cycle detected",
        ));
    }
    Ok(findings)
}

pub(in crate::policy) fn health(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let ui = root.join("ui");
    let provider = TypeScriptProvider::new(&ui.join("tsconfig.json"));
    let mut files = Vec::new();
    collect_source_files(&ui.join("src"), &mut files)?;
    let mut findings = Vec::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if ratchet.generated.iter().any(|entry| entry.path == relative) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let analysis = provider.analyze(&path, &source);
        if !analysis.findings.is_empty() {
            findings.extend(analysis.findings);
            continue;
        }
        let is_route = relative.contains("/routes/");
        let is_test = relative.contains("/__tests__/") || relative.contains("/tests/");
        let target = if is_route {
            ratchet.budgets.typescript.route_file_lines
        } else if is_test {
            ratchet.budgets.typescript.test_file_lines
        } else {
            ratchet.budgets.typescript.module_lines
        };
        let logical_lines = source
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        if logical_lines > target {
            findings.push(Finding::warning(
                "health.typescript.file-lines",
                &relative,
                1,
                &format!("{logical_lines} logical lines exceeds target {target}"),
                "split the module by responsibility; required ratchets are shrink-only",
                "cargo xtask health",
            ));
        }
        for function in analysis.function_spans {
            for (rule, value, target) in [
                (
                    "health.typescript.function-lines",
                    function.lines,
                    ratchet.budgets.typescript.function_lines,
                ),
                (
                    "health.typescript.function-cyclomatic-complexity",
                    function.cyclomatic,
                    ratchet.budgets.typescript.cyclomatic_complexity,
                ),
                (
                    "health.typescript.function-cognitive-complexity",
                    function.cognitive,
                    ratchet.budgets.typescript.cognitive_complexity,
                ),
            ] {
                if value > target {
                    findings.push(Finding::warning(
                        rule,
                        &relative,
                        function.line,
                        &format!("function measurement {value} exceeds target {target}"),
                        "reduce function structure without refreshing the baseline",
                        "cargo xtask health",
                    ));
                }
            }
        }
        for (rule, value) in [
            ("health.typescript.suppressions", analysis.suppressions),
            ("health.typescript.assertions", analysis.assertions),
        ] {
            if value > 0 {
                findings.push(Finding::warning(
                    rule,
                    &relative,
                    1,
                    &format!("count {value} exceeds target 0"),
                    "reduce the scoped presence count and lower its ratchet",
                    "cargo xtask health",
                ));
            }
        }
    }
    Ok(findings)
}
