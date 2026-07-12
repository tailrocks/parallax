use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use oxc_allocator::Allocator;
use oxc_ast::{AstKind, ast::Expression};
use oxc_parser::Parser;
use oxc_resolver::{
    ResolveOptions, Resolver, TsconfigDiscovery, TsconfigOptions, TsconfigReferences,
};
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};

use crate::diagnostic::Finding;

use super::config::Ratchet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportEdge {
    pub specifier: String,
    pub resolved: PathBuf,
    pub type_only: bool,
    pub dynamic: bool,
    pub line: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metrics {
    pub functions: usize,
    pub jsx_elements: usize,
    pub directives: usize,
    pub exports: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Analysis {
    pub imports: Vec<ImportEdge>,
    pub metrics: Metrics,
    pub findings: Vec<Finding>,
    function_spans: Vec<(usize, usize)>,
}

pub struct TypeScriptProvider {
    resolver: Resolver,
}

pub fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let ui = root.join("ui");
    let provider = TypeScriptProvider::new(&ui.join("tsconfig.json"));
    let mut files = Vec::new();
    collect_source_files(&ui.join("src"), &mut files)?;
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
        for edge in analysis.imports {
            if !edge.resolved.starts_with(&ui) {
                continue;
            }
            graph
                .entry(path.clone())
                .or_default()
                .push(edge.resolved.clone());
            check_boundary(&ui, &path, &edge, &mut findings);
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

pub fn health(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let ui = root.join("ui");
    let provider = TypeScriptProvider::new(&ui.join("tsconfig.json"));
    let mut files = Vec::new();
    collect_source_files(&ui.join("src"), &mut files)?;
    let mut findings = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let analysis = provider.analyze(&path, &source);
        if !analysis.findings.is_empty() {
            findings.extend(analysis.findings);
            continue;
        }
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
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
        for (line, lines) in analysis
            .function_spans
            .into_iter()
            .filter(|(_, lines)| *lines > ratchet.budgets.typescript.function_lines)
        {
            findings.push(Finding::warning(
                "health.typescript.function-lines",
                &relative,
                line,
                &format!(
                    "function has {lines} lines, target {}",
                    ratchet.budgets.typescript.function_lines
                ),
                "extract focused behavior without refreshing the baseline",
                "cargo xtask health",
            ));
        }
    }
    Ok(findings)
}

impl TypeScriptProvider {
    pub fn new(tsconfig: &Path) -> Self {
        Self {
            resolver: Resolver::new(ResolveOptions {
                condition_names: vec!["browser".into(), "import".into(), "default".into()],
                extensions: vec![
                    ".ts".into(),
                    ".tsx".into(),
                    ".js".into(),
                    ".jsx".into(),
                    ".json".into(),
                ],
                extension_alias: vec![
                    (
                        ".js".into(),
                        vec![".ts".into(), ".tsx".into(), ".js".into()],
                    ),
                    (".jsx".into(), vec![".tsx".into(), ".jsx".into()]),
                ],
                tsconfig: Some(TsconfigDiscovery::Manual(TsconfigOptions {
                    config_file: tsconfig.to_path_buf(),
                    references: TsconfigReferences::Auto,
                })),
                alias_fields: vec![vec!["browser".into()]],
                main_fields: vec!["browser".into(), "module".into(), "main".into()],
                module_type: true,
                ..ResolveOptions::default()
            }),
        }
    }

    pub fn analyze(&self, path: &Path, source: &str) -> Analysis {
        let mut analysis = Analysis::default();
        let source_type = match SourceType::from_path(path) {
            Ok(source_type) => source_type,
            Err(error) => {
                analysis.findings.push(finding(
                    path,
                    1,
                    "typescript.source-type",
                    &error.to_string(),
                ));
                return analysis;
            }
        };
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, source_type).parse();
        for diagnostic in &parsed.diagnostics {
            analysis.findings.push(finding(
                path,
                1,
                "typescript.parse",
                &diagnostic.to_string(),
            ));
        }
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            return analysis;
        }

        for (specifier, requests) in &parsed.module_record.requested_modules {
            let line = requests
                .first()
                .map_or(1, |request| line_at(source, request.span.start));
            let type_only = requests.iter().all(|request| request.is_type);
            self.resolve(
                path,
                specifier.as_str(),
                type_only,
                false,
                line,
                &mut analysis,
            );
        }

        let semantic = SemanticBuilder::new_compiler()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(&parsed.program);
        for diagnostic in &semantic.diagnostics {
            analysis.findings.push(finding(
                path,
                1,
                "typescript.semantic",
                &diagnostic.to_string(),
            ));
        }
        if !semantic.diagnostics.is_empty() {
            return analysis;
        }
        for node in semantic.semantic.nodes().iter() {
            match node.kind() {
                AstKind::Function(function) => {
                    analysis.metrics.functions += 1;
                    analysis
                        .function_spans
                        .push(span_lines(source, function.span()));
                }
                AstKind::ArrowFunctionExpression(function) => {
                    analysis.metrics.functions += 1;
                    analysis
                        .function_spans
                        .push(span_lines(source, function.span()));
                }
                AstKind::JSXElement(_) => analysis.metrics.jsx_elements += 1,
                AstKind::Directive(_) => analysis.metrics.directives += 1,
                AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportDefaultDeclaration(_)
                | AstKind::ExportAllDeclaration(_) => analysis.metrics.exports += 1,
                AstKind::ImportExpression(expression) => {
                    let line = line_at(source, expression.span().start);
                    if let Expression::StringLiteral(literal) = &expression.source {
                        self.resolve(
                            path,
                            literal.value.as_str(),
                            false,
                            true,
                            line,
                            &mut analysis,
                        );
                    } else {
                        analysis.findings.push(finding(
                            path,
                            line,
                            "typescript.dynamic-import",
                            "dynamic import target is not a string literal",
                        ));
                    }
                }
                _ => {}
            }
        }
        analysis
    }

    fn resolve(
        &self,
        path: &Path,
        specifier: &str,
        type_only: bool,
        dynamic: bool,
        line: usize,
        analysis: &mut Analysis,
    ) {
        match self.resolver.resolve_file(path, specifier) {
            Ok(resolution) => analysis.imports.push(ImportEdge {
                specifier: specifier.into(),
                resolved: resolution.into_path_buf(),
                type_only,
                dynamic,
                line,
            }),
            Err(error) => analysis.findings.push(finding(
                path,
                line,
                "typescript.resolve",
                &format!("cannot resolve `{specifier}`: {error}"),
            )),
        }
    }
}

fn collect_source_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read directory {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files)?;
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("ts" | "tsx" | "js" | "jsx")
        ) {
            files.push(path);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Layer {
    App,
    Layout,
    Routes,
    Features,
    Domain,
    Platform,
    Shared,
    Test,
    Generated,
}

fn layer(ui: &Path, path: &Path) -> Layer {
    let relative = path.strip_prefix(ui).unwrap_or(path);
    let text = relative.to_string_lossy().replace('\\', "/");
    if text == "src/routeTree.gen.ts" {
        return Layer::Generated;
    }
    if text.contains("/__tests__/")
        || text.contains("/tests/")
        || text.starts_with("src/test/")
        || text.starts_with("tests/")
    {
        return Layer::Test;
    }
    if text.starts_with("src/routes/") {
        return Layer::Routes;
    }
    if text.starts_with("src/features/") {
        return Layer::Features;
    }
    if text.starts_with("src/domain/") {
        return Layer::Domain;
    }
    if text.starts_with("src/platform/") {
        return Layer::Platform;
    }
    if text.starts_with("src/layout/") {
        return Layer::Layout;
    }
    if text.starts_with("src/shared/")
        || text.starts_with("src/components/")
        || text.starts_with("src/hooks/")
        || text.starts_with("src/lib/")
    {
        return Layer::Shared;
    }
    Layer::App
}

fn check_boundary(ui: &Path, source: &Path, edge: &ImportEdge, findings: &mut Vec<Finding>) {
    let from = layer(ui, source);
    let to = layer(ui, &edge.resolved);
    if from != Layer::Test && to == Layer::Test {
        findings.push(finding(
            source,
            edge.line,
            "typescript.test-boundary",
            "production module imports test support",
        ));
    }
    let source_name = source.to_string_lossy();
    let target_name = edge.resolved.to_string_lossy();
    if !source_name.contains(".server.") && target_name.contains(".server.") {
        findings.push(finding(
            source,
            edge.line,
            "typescript.server-boundary",
            "browser-reachable module imports a server-only module",
        ));
    }
    if source_name.contains(".server.") && target_name.contains(".client.") {
        findings.push(finding(
            source,
            edge.line,
            "typescript.client-boundary",
            "server-only module imports a client-only module",
        ));
    }
    let allowed = match from {
        Layer::App | Layer::Test | Layer::Generated => true,
        Layer::Routes | Layer::Layout => matches!(
            to,
            Layer::Features | Layer::Domain | Layer::Shared | Layer::Generated
        ),
        Layer::Features => matches!(
            to,
            Layer::Features | Layer::Domain | Layer::Platform | Layer::Shared
        ),
        Layer::Platform => matches!(to, Layer::Domain | Layer::Shared),
        Layer::Domain => to == Layer::Shared,
        Layer::Shared => to == Layer::Shared,
    };
    if !allowed {
        findings.push(finding(
            source,
            edge.line,
            "typescript.layer",
            &format!(
                "forbidden {from:?} -> {to:?} import through `{}`",
                edge.specifier
            ),
        ));
    }
}

fn find_cycle(graph: &BTreeMap<PathBuf, Vec<PathBuf>>) -> Option<PathBuf> {
    fn visit(
        node: &PathBuf,
        graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
        active: &mut BTreeSet<PathBuf>,
        done: &mut BTreeSet<PathBuf>,
    ) -> Option<PathBuf> {
        if active.contains(node) {
            return Some(node.clone());
        }
        if !done.insert(node.clone()) {
            return None;
        }
        active.insert(node.clone());
        for target in graph.get(node).into_iter().flatten() {
            if let Some(cycle) = visit(target, graph, active, done) {
                return Some(cycle);
            }
        }
        active.remove(node);
        None
    }
    let mut done = BTreeSet::new();
    for node in graph.keys() {
        if let Some(cycle) = visit(node, graph, &mut BTreeSet::new(), &mut done) {
            return Some(cycle);
        }
    }
    None
}

fn line_at(source: &str, offset: u32) -> usize {
    source.as_bytes()[..offset as usize]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

fn span_lines(source: &str, span: oxc_span::Span) -> (usize, usize) {
    let start = line_at(source, span.start);
    let end = line_at(source, span.end);
    (start, end.saturating_sub(start) + 1)
}

fn finding(path: &Path, line: usize, rule: &str, reason: &str) -> Finding {
    Finding::error(
        rule,
        &path.to_string_lossy(),
        line,
        reason,
        "correct the TypeScript syntax or module target",
        "cargo xtask policy --only typescript",
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn fixture() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("parallax-oxc-{id}"));
        fs::create_dir_all(root.join("src/lib")).expect("fixture directories should be created");
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
        )
        .expect("tsconfig should be written");
        fs::write(root.join("src/lib/value.ts"), "export const value = 1")
            .expect("module should be written");
        root
    }

    #[test]
    fn parses_tsx_alias_type_reexport_and_dynamic_import() {
        let root = fixture();
        let path = root.join("src/index.tsx");
        let source = "'use client'\nimport type { T } from './types'\nexport { value } from '@/lib/value'\nconst C = () => <div />\nvoid import('./lazy')";
        fs::write(root.join("src/types.ts"), "export type T = string")
            .expect("type module should be written");
        fs::write(root.join("src/lazy.ts"), "export default 1")
            .expect("lazy module should be written");
        let analysis = TypeScriptProvider::new(&root.join("tsconfig.json")).analyze(&path, source);
        assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
        assert_eq!(analysis.imports.len(), 3);
        assert!(analysis.imports.iter().any(|edge| edge.type_only));
        assert!(analysis.imports.iter().any(|edge| edge.dynamic));
        assert_eq!(analysis.metrics.functions, 1);
        assert_eq!(analysis.metrics.jsx_elements, 1);
        assert_eq!(analysis.metrics.directives, 1);
        assert_eq!(analysis.metrics.exports, 1);
        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[test]
    fn fails_closed_on_parse_resolution_and_nonliteral_dynamic_import() {
        let root = fixture();
        let provider = TypeScriptProvider::new(&root.join("tsconfig.json"));
        let parse = provider.analyze(&root.join("src/bad.ts"), "const =");
        assert!(
            parse
                .findings
                .iter()
                .any(|finding| finding.rule_id == "typescript.parse")
        );
        let resolve = provider.analyze(
            &root.join("src/missing.ts"),
            "import './absent'; import(name)",
        );
        assert!(
            resolve
                .findings
                .iter()
                .any(|finding| finding.rule_id == "typescript.resolve")
        );
        assert!(
            resolve
                .findings
                .iter()
                .any(|finding| finding.rule_id == "typescript.dynamic-import")
        );
        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[test]
    fn resolves_extension_index_and_package_exports_for_js_and_jsx() {
        let root = fixture();
        fs::write(root.join("src/lib/index.ts"), "export const indexed = 1")
            .expect("index module should be written");
        fs::create_dir_all(root.join("node_modules/pkg/dist"))
            .expect("package directories should be created");
        fs::write(
            root.join("node_modules/pkg/package.json"),
            r#"{"name":"pkg","exports":{".":"./dist/index.js"}}"#,
        )
        .expect("package manifest should be written");
        fs::write(
            root.join("node_modules/pkg/dist/index.js"),
            "export const pkg = 1",
        )
        .expect("package module should be written");
        let provider = TypeScriptProvider::new(&root.join("tsconfig.json"));
        for (name, source) in [
            ("entry.js", "import { pkg } from 'pkg'; export { pkg }"),
            (
                "view.jsx",
                "import { indexed } from '@/lib'; export const V = () => <p>{indexed}</p>",
            ),
        ] {
            let path = root.join("src").join(name);
            let analysis = provider.analyze(&path, source);
            assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
            assert_eq!(analysis.imports.len(), 1);
        }
        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[test]
    fn parity_corpus_names_every_downstream_case() {
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/typescript-import-parity.json"))
                .expect("parity corpus should be valid JSON");
        let covers: BTreeSet<_> = corpus["cases"]
            .as_array()
            .expect("cases should be an array")
            .iter()
            .flat_map(|case| {
                case["covers"]
                    .as_array()
                    .expect("covers should be an array")
            })
            .map(|value| value.as_str().expect("coverage value should be a string"))
            .collect();
        for required in [
            "ts",
            "tsx",
            "js",
            "jsx",
            "extension",
            "index-lookup",
            "alias",
            "type-only",
            "dynamic",
            "reexport",
            "barrel",
            "package-exports",
            "resolution-failure",
            "cycle",
            "app",
            "layout",
            "routes",
            "features",
            "domain",
            "platform",
            "shared",
            "feature-facade",
            "feature-deep-import",
            "source-tests",
            "src-test-harness",
            "e2e",
            "server",
            "client",
            "generated-route-composition",
        ] {
            assert!(covers.contains(required), "missing parity case: {required}");
        }
    }

    #[test]
    fn enforces_every_layer_and_test_runtime_boundary() {
        let root = Path::new("/repo/ui");
        let cases = [
            ("src/shared/a.ts", "src/routes/b.ts", "typescript.layer"),
            ("src/domain/a.ts", "src/platform/b.ts", "typescript.layer"),
            ("src/platform/a.ts", "src/features/b.ts", "typescript.layer"),
            (
                "src/routes/a.ts",
                "src/test/b.ts",
                "typescript.test-boundary",
            ),
            (
                "src/routes/a.ts",
                "src/lib/b.server.ts",
                "typescript.server-boundary",
            ),
            (
                "src/lib/a.server.ts",
                "src/lib/b.client.ts",
                "typescript.client-boundary",
            ),
        ];
        for (source, target, rule) in cases {
            let mut findings = Vec::new();
            check_boundary(
                root,
                &root.join(source),
                &ImportEdge {
                    specifier: "fixture".into(),
                    resolved: root.join(target),
                    type_only: false,
                    dynamic: false,
                    line: 1,
                },
                &mut findings,
            );
            assert!(findings.iter().any(|finding| finding.rule_id == rule));
        }
    }
}
