use std::fs;

use super::*;

fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary directory");
    let root = directory.path();
    fs::create_dir_all(root.join("src/lib")).expect("fixture directories should be created");
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    )
    .expect("tsconfig should be written");
    fs::write(root.join("src/lib/value.ts"), "export const value = 1")
        .expect("module should be written");
    directory
}

#[test]
fn parses_tsx_alias_type_reexport_and_dynamic_import() {
    let directory = fixture();
    let root = directory.path();
    let path = root.join("src/index.tsx");
    let source = "'use client'\nimport type { T } from './types'\nexport { value } from '@/lib/value'\nconst C = () => <div />\nvoid import('./lazy')";
    fs::write(root.join("src/types.ts"), "export type T = string")
        .expect("type module should be written");
    fs::write(root.join("src/lazy.ts"), "export default 1").expect("lazy module should be written");
    let analysis = TypeScriptProvider::new(&root.join("tsconfig.json")).analyze(&path, source);
    assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
    assert_eq!(analysis.imports.len(), 3);
    assert!(analysis.imports.iter().any(|edge| edge.type_only));
    assert!(analysis.imports.iter().any(|edge| edge.dynamic));
    assert_eq!(analysis.metrics.functions, 1);
    assert_eq!(analysis.metrics.jsx_elements, 1);
    assert_eq!(analysis.metrics.directives, 1);
    assert_eq!(analysis.metrics.exports, 1);
}

#[test]
fn fails_closed_on_parse_resolution_and_nonliteral_dynamic_import() {
    let directory = fixture();
    let root = directory.path();
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
}

#[test]
fn resolves_extension_index_and_package_exports_for_js_and_jsx() {
    let directory = fixture();
    let root = directory.path();
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
}

#[test]
fn resolves_browser_and_server_package_export_conditions() {
    let directory = fixture();
    let root = directory.path();
    fs::create_dir_all(root.join("node_modules/pkg/dist"))
        .expect("package directories should be created");
    fs::write(
        root.join("node_modules/pkg/package.json"),
        r#"{"name":"pkg","exports":{".":{"browser":"./dist/browser.js","node":"./dist/node.js","default":"./dist/default.js"}}}"#,
    )
    .expect("package manifest should be written");
    for target in ["browser.js", "node.js", "default.js"] {
        fs::write(
            root.join("node_modules/pkg/dist").join(target),
            "export default 1",
        )
        .expect("package target should be written");
    }
    let provider = TypeScriptProvider::new(&root.join("tsconfig.json"));
    for (name, expected) in [
        ("entry.client.ts", "browser.js"),
        ("entry.server.ts", "node.js"),
    ] {
        let analysis = provider.analyze(&root.join("src").join(name), "import value from 'pkg'");
        assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
        assert_eq!(
            analysis.imports[0]
                .resolved
                .file_name()
                .and_then(|value| value.to_str()),
            Some(expected)
        );
    }
}

#[test]
fn parity_corpus_names_every_downstream_case() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/typescript-import-parity.json"
    ))
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

#[test]
fn measures_nested_ast_complexity_and_presence() {
    let directory = fixture();
    let root = directory.path();
    let path = root.join("src/complex.tsx");
    let source = "// @ts-expect-error fixture\nconst f = () => { if (a && b) { for (const x of xs) { expect(x) } } }";
    let analysis = TypeScriptProvider::new(&root.join("tsconfig.json")).analyze(&path, source);
    assert!(analysis.findings.is_empty());
    assert!(analysis.function_spans[0].cyclomatic > 2);
    assert!(analysis.function_spans[0].cognitive > 2);
    assert_eq!(analysis.suppressions, 1);
    assert_eq!(analysis.assertions, 1);
}
