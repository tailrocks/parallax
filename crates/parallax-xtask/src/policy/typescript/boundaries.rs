//! Import resolution, layer boundaries, and cycle detection.

use super::*;

pub(super) fn resolver(
    tsconfig: &Path,
    condition_names: Vec<String>,
    alias_fields: Vec<Vec<String>>,
    main_fields: Vec<String>,
) -> Resolver {
    Resolver::new(ResolveOptions {
        condition_names,
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
        alias_fields,
        main_fields,
        module_type: true,
        ..ResolveOptions::default()
    })
}

pub(super) fn collect_source_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
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
pub(super) enum Layer {
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

pub(super) fn layer(ui: &Path, path: &Path) -> Layer {
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

pub(super) fn check_boundary(
    ui: &Path,
    source: &Path,
    edge: &ImportEdge,
    findings: &mut Vec<Finding>,
) {
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
    let source_feature = feature_name(ui, source);
    let target_feature = feature_name(ui, &edge.resolved);
    if let Some(target_feature) = target_feature
        && source_feature.as_deref() != Some(target_feature.as_str())
    {
        let expected = ui
            .join("src/features")
            .join(&target_feature)
            .join("index.ts");
        if edge.resolved != expected {
            findings.push(finding(source, edge.line, "typescript.feature-facade", &format!("external import reaches inside feature `{target_feature}` instead of its index.ts facade")));
        }
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

pub(super) fn feature_name(ui: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(ui).ok()?;
    let mut parts = relative.components();
    if parts.next()?.as_os_str() != "src" || parts.next()?.as_os_str() != "features" {
        return None;
    }
    parts
        .next()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
}

pub(super) fn find_cycle(graph: &BTreeMap<PathBuf, Vec<PathBuf>>) -> Option<PathBuf> {
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
