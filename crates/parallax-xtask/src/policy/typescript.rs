use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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

mod boundaries;
mod health_metrics;
pub(super) mod packages;
mod workspace;

use boundaries::*;
use health_metrics::*;
pub(super) use workspace::{check_workspace, health};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ImportEdge {
    pub specifier: String,
    pub resolved: PathBuf,
    pub type_only: bool,
    pub dynamic: bool,
    pub line: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Metrics {
    pub functions: usize,
    pub jsx_elements: usize,
    pub directives: usize,
    pub exports: usize,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Analysis {
    pub imports: Vec<ImportEdge>,
    pub metrics: Metrics,
    pub findings: Vec<Finding>,
    function_spans: Vec<FunctionHealth>,
    suppressions: usize,
    assertions: usize,
    star_exports: usize,
}

#[derive(Clone, Debug, Default)]
struct FunctionHealth {
    line: usize,
    lines: usize,
    cyclomatic: usize,
    cognitive: usize,
}

pub(super) struct TypeScriptProvider {
    browser_resolver: Resolver,
    server_resolver: Resolver,
}

impl TypeScriptProvider {
    pub(super) fn new(tsconfig: &Path) -> Self {
        Self {
            browser_resolver: resolver(
                tsconfig,
                vec!["browser".into(), "import".into(), "default".into()],
                vec![vec!["browser".into()]],
                vec!["browser".into(), "module".into(), "main".into()],
            ),
            server_resolver: resolver(
                tsconfig,
                vec!["node".into(), "import".into(), "default".into()],
                Vec::new(),
                vec!["module".into(), "main".into()],
            ),
        }
    }

    pub(super) fn analyze(&self, path: &Path, source: &str) -> Analysis {
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
        let nodes = semantic.semantic.nodes();
        let mut complexity = HashMap::new();
        for node in nodes.iter() {
            if matches!(
                node.kind(),
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
            ) {
                complexity.insert(node.id(), (1_usize, 0_usize));
            }
        }
        for node in nodes.iter().filter(|node| branch(node.kind())) {
            let mut nesting = 0;
            for ancestor in nodes.ancestors(node.id()) {
                if matches!(
                    ancestor.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                ) {
                    if let Some((cyclomatic, cognitive)) = complexity.get_mut(&ancestor.id()) {
                        *cyclomatic += 1;
                        *cognitive += 1 + nesting;
                    }
                    break;
                }
                if branch(ancestor.kind()) {
                    nesting += 1;
                }
            }
        }
        analysis.suppressions = semantic
            .semantic
            .comments()
            .iter()
            .filter(|comment| {
                let text = &source[comment.span.start as usize..comment.span.end as usize];
                text.contains("@ts-") || text.contains("eslint-disable")
            })
            .count();
        for node in nodes.iter() {
            match node.kind() {
                AstKind::Function(function) => {
                    analysis.metrics.functions += 1;
                    analysis.function_spans.push(function_health(
                        source,
                        function.span(),
                        complexity[&node.id()],
                    ));
                }
                AstKind::ArrowFunctionExpression(function) => {
                    analysis.metrics.functions += 1;
                    analysis.function_spans.push(function_health(
                        source,
                        function.span(),
                        complexity[&node.id()],
                    ));
                }
                AstKind::CallExpression(call)
                    if call
                        .callee
                        .get_identifier_reference()
                        .is_some_and(|identifier| {
                            matches!(
                                identifier.name.as_str(),
                                "expect" | "assert" | "assertEquals"
                            )
                        }) =>
                {
                    analysis.assertions += 1;
                }
                AstKind::JSXElement(_) => analysis.metrics.jsx_elements += 1,
                AstKind::Directive(_) => analysis.metrics.directives += 1,
                AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportDefaultDeclaration(_)
                | AstKind::ExportAllDeclaration(_) => {
                    analysis.metrics.exports += 1;
                    if matches!(node.kind(), AstKind::ExportAllDeclaration(_)) {
                        analysis.star_exports += 1;
                    }
                }
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
        let resolver = if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains(".server."))
        {
            &self.server_resolver
        } else {
            &self.browser_resolver
        };
        match resolver.resolve_file(path, specifier) {
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
mod tests;
