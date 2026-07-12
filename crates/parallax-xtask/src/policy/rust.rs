use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use proc_macro2::Span;
use rustc_lexer::{FrontmatterAllowed, TokenKind};
use syn::{spanned::Spanned, visit::Visit};

use crate::diagnostic::Finding;

use super::config::Ratchet;

#[derive(Debug, Eq, PartialEq)]
struct FunctionMetric {
    line: usize,
    lines: usize,
    name: String,
}

#[derive(Debug, Eq, PartialEq)]
struct FileMetric {
    logical_lines: usize,
    functions: Vec<FunctionMetric>,
}

pub fn health(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut files = Vec::new();
    collect(&root.join("crates"), &mut files)?;
    let mut findings = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let metric =
            analyze(&source).with_context(|| format!("failed to parse {}", path.display()))?;
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        let is_root = matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("lib.rs" | "main.rs")
        );
        let is_test = relative.contains("/tests/") || relative.ends_with("/tests.rs");
        let target = if is_root {
            ratchet.budgets.rust.root_file_lines
        } else if is_test {
            ratchet.budgets.rust.test_file_lines
        } else {
            ratchet.budgets.rust.production_file_lines
        };
        if metric.logical_lines > target {
            findings.push(Finding::warning(
                "health.rust.file-lines",
                &relative,
                1,
                &format!(
                    "{} logical lines exceeds target {target}",
                    metric.logical_lines
                ),
                "split the file by responsibility; required ratchets are shrink-only",
                "cargo xtask health",
            ));
        }
        for function in metric
            .functions
            .into_iter()
            .filter(|function| function.lines > ratchet.budgets.rust.function_lines)
        {
            findings.push(Finding::warning(
                "health.rust.function-lines",
                &relative,
                function.line,
                &format!(
                    "{} has {} lines, target {}",
                    function.name, function.lines, ratchet.budgets.rust.function_lines
                ),
                "extract focused operations without refreshing the baseline",
                "cargo xtask health",
            ));
        }
    }
    Ok(findings)
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn analyze(source: &str) -> Result<FileMetric> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = FunctionVisitor::default();
    visitor.visit_file(&syntax);
    Ok(FileMetric {
        logical_lines: logical_lines(source),
        functions: visitor.functions,
    })
}

fn logical_lines(source: &str) -> usize {
    let mut offset = 0;
    let mut line = 0;
    let mut occupied = vec![false; source.bytes().filter(|byte| *byte == b'\n').count() + 1];
    for token in rustc_lexer::tokenize(source, FrontmatterAllowed::Yes) {
        let end = offset + token.len as usize;
        let newlines = source[offset..end]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        if !matches!(
            token.kind,
            TokenKind::Whitespace | TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        ) {
            for occupied_line in line..=(line + newlines).min(occupied.len() - 1) {
                occupied[occupied_line] = true;
            }
        }
        line += newlines;
        offset = end;
    }
    occupied.into_iter().filter(|occupied| *occupied).count()
}

#[derive(Default)]
struct FunctionVisitor {
    functions: Vec<FunctionMetric>,
    closures: usize,
}

impl FunctionVisitor {
    fn push(&mut self, name: String, span: Span) {
        let start = span.start().line;
        let end = span.end().line;
        self.functions.push(FunctionMetric {
            line: start,
            lines: end.saturating_sub(start) + 1,
            name,
        });
    }
}

impl<'ast> Visit<'ast> for FunctionVisitor {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.push(function.sig.ident.to_string(), function.span());
        syn::visit::visit_item_fn(self, function);
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.push(function.sig.ident.to_string(), function.span());
        syn::visit::visit_impl_item_fn(self, function);
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.closures += 1;
        self.push(format!("closure#{}", self.closures), closure.span());
        syn::visit::visit_expr_closure(self, closure);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_functions_closures_and_comment_free_logical_lines() {
        let source = "// comment\nfn work() {\n  let f = || {\n    1\n  };\n}\n\n/* ignored */\n";
        let metric = analyze(source).expect("fixture should parse");
        assert_eq!(metric.logical_lines, 5);
        assert_eq!(metric.functions.len(), 2);
        assert_eq!(metric.functions[0].name, "work");
    }

    #[test]
    fn malformed_rust_fails_closed() {
        assert!(analyze("fn {").is_err());
    }
}
