use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use syn::{
    Attribute, ExprMethodCall, ImplItemFn, ItemFn, ItemMod, LitStr, Macro, Meta, Path as SynPath,
    visit::Visit,
};

use crate::diagnostic::Finding;

use super::config::Ratchet;

mod anyhow_edges;
mod toolchain;

pub(super) fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    check_cargo(root, &mut findings)?;
    check_bun(root, &mut findings)?;
    check_native_tables(root, &mut findings)?;
    check_composition(root, &mut findings)?;
    check_clone_floors(root, ratchet, &mut findings)?;
    anyhow_edges::check(root, ratchet, &mut findings)?;
    check_ingest_logging(root, &mut findings)?;
    check_self_telemetry(root, &mut findings)?;
    Ok(findings)
}

fn check_cargo(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let metadata = MetadataCommand::new().current_dir(root).exec()?;
    toolchain::check(root, &metadata, findings)?;
    let workspace: BTreeSet<_> = metadata.workspace_members.iter().collect();
    for package in metadata
        .packages
        .iter()
        .filter(|package| workspace.contains(&package.id))
    {
        if package.license.as_deref() != Some("Apache-2.0") {
            findings.push(error(
                "product.license",
                &package.manifest_path,
                "workspace package license is not Apache-2.0",
            ));
        }
    }
    let active: BTreeSet<_> = metadata
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    for forbidden in ["rustls", "webpki", "rustls-webpki"] {
        if active.contains(forbidden) {
            findings.push(error(
                "product.tls",
                Path::new("Cargo.lock"),
                &format!("forbidden TLS backend `{forbidden}` is active"),
            ));
        }
    }
    let greptime = metadata
        .packages
        .iter()
        .find(|package| package.name == "parallax-greptime");
    let metadata_adapter = metadata
        .packages
        .iter()
        .find(|package| package.name == "parallax-metadata");
    let has_turso = metadata_adapter.is_some_and(|package| {
        package
            .dependencies
            .iter()
            .any(|dependency| dependency.name == "turso")
    });
    let has_greptime_transport = greptime.is_some_and(|package| {
        package
            .dependencies
            .iter()
            .any(|dependency| dependency.name == "reqwest")
    });
    let has_storage_stack = has_turso && has_greptime_transport;
    if !has_storage_stack {
        findings.push(error(
            "product.storage-stack",
            Path::new("Cargo.toml"),
            "metadata ownership must compose Turso and Greptime ownership must compose HTTP transport",
        ));
    }
    let reqwest = greptime.and_then(|package| {
        package
            .dependencies
            .iter()
            .find(|dependency| dependency.name == "reqwest")
    });
    if !reqwest.is_some_and(|dependency| {
        !dependency.uses_default_features
            && dependency
                .features
                .iter()
                .any(|feature| feature == "native-tls")
    }) {
        findings.push(error(
            "product.tls",
            Path::new("Cargo.toml"),
            "reqwest must disable defaults and enable host native-tls",
        ));
    }
    Ok(())
}

fn check_bun(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let package: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("ui/package.json"))?)?;
    let scripts = package["scripts"]
        .as_object()
        .context("ui package scripts must be an object")?;
    for (name, command) in scripts {
        let command = command
            .as_str()
            .context("package script must be a string")?;
        if matches!(
            name.as_str(),
            "dev"
                | "build"
                | "preview"
                | "test"
                | "test:ci"
                | "lint"
                | "format"
                | "check"
                | "typecheck"
        ) && !command.starts_with("bunx --bun --no-install ")
            && !command.starts_with("bun ")
        {
            findings.push(error(
                "product.bun",
                Path::new("ui/package.json"),
                &format!("script `{name}` is not lock-local Bun execution"),
            ));
        }
    }
    let bunfig: toml::Value = toml::from_str(&fs::read_to_string(root.join("ui/bunfig.toml"))?)?;
    if bunfig["run"]["bun"].as_bool() != Some(true)
        || bunfig["install"]["auto"].as_str() != Some("disable")
    {
        findings.push(error(
            "product.bun",
            Path::new("ui/bunfig.toml"),
            "Bun runtime or auto-install policy is missing",
        ));
    }
    for name in [
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "pnpm-workspace.yaml",
        ".npmrc",
    ] {
        if root.join(name).exists() || root.join("ui").join(name).exists() {
            findings.push(error(
                "product.bun",
                Path::new(name),
                "foreign package-manager artifact exists",
            ));
        }
    }
    Ok(())
}

fn check_native_tables(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join("crates/parallax-greptime/src/greptime");
    let literals = string_literals_in(&path)?;
    for table in ["opentelemetry_traces", "opentelemetry_logs"] {
        if !literals.iter().any(|literal| literal.contains(table)) {
            findings.push(error(
                "product.native-tables",
                &path,
                &format!("native table `{table}` is not referenced"),
            ));
        }
    }
    Ok(())
}

fn string_literals_in(directory: &Path) -> Result<Vec<String>> {
    let mut literals = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            literals.extend(string_literals_in(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            literals.extend(string_literals(&path)?);
        }
    }
    Ok(literals)
}

fn check_composition(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    for relative in [
        "crates/parallax-cli/src/main.rs",
        "crates/parallax-server/src/serve.rs",
        "crates/parallax-server/src/lib.rs",
    ] {
        let path = root.join(relative);
        let syntax = parse(&path)?;
        let mut visitor = IdentifierVisitor::default();
        visitor.visit_file(&syntax);
        if visitor.identifiers.contains("MemoryStore") {
            findings.push(error(
                "product.memory-store",
                &path,
                "release composition references MemoryStore",
            ));
        }
    }
    Ok(())
}

fn check_clone_floors(root: &Path, ratchet: &Ratchet, findings: &mut Vec<Finding>) -> Result<()> {
    for floor in &ratchet.product.clone_floors {
        let path = root.join(&floor.path);
        let syntax = parse(&path)?;
        let mut visitor = CloneVisitor::default();
        visitor.visit_file(&syntax);
        if visitor.count > floor.ceiling {
            findings.push(error(
                "product.clone-floor",
                &path,
                &format!(
                    "clone count {} exceeds ceiling {}",
                    visitor.count, floor.ceiling
                ),
            ));
        } else if visitor.count < floor.ceiling {
            findings.push(error(
                "product.clone-floor.stale",
                &path,
                &format!(
                    "clone count shrank to {}; lower the ratchet ceiling {}",
                    visitor.count, floor.ceiling
                ),
            ));
        }
    }
    Ok(())
}

fn check_ingest_logging(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    for relative in [
        "crates/parallax-storage/src/adapter.rs",
        "crates/parallax-greptime/src/greptime.rs",
        "crates/parallax-metadata/src/turso.rs",
    ] {
        let path = root.join(relative);
        let syntax = parse(&path)?;
        let mut visitor = IngestLogVisitor::default();
        visitor.visit_file(&syntax);
        for name in visitor.violations {
            findings.push(error(
                "product.ingest-log",
                &path,
                &format!(
                    "ingest function `{name}` emits tracing and can create a self-telemetry loop"
                ),
            ));
        }
    }
    Ok(())
}

fn check_self_telemetry(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join("crates/parallax-server/src/self_telemetry.rs");
    let literals = string_literals(&path)?;
    for value in ["PARALLAX_SELF_OTLP", "off"] {
        if !literals.iter().any(|literal| literal == value) {
            findings.push(error(
                "product.self-telemetry",
                &path,
                &format!("self-telemetry control `{value}` lacks source/test coverage"),
            ));
        }
    }
    Ok(())
}

fn parse(path: &Path) -> Result<syn::File> {
    syn::parse_file(&fs::read_to_string(path)?)
        .with_context(|| format!("failed to parse {}", path.display()))
}

fn string_literals(path: &Path) -> Result<Vec<String>> {
    let syntax = parse(path)?;
    let mut visitor = LiteralVisitor::default();
    visitor.visit_file(&syntax);
    Ok(visitor.values)
}

pub(super) fn error(rule: &str, path: impl AsRef<Path>, reason: &str) -> Finding {
    Finding::error(
        rule,
        &path.as_ref().to_string_lossy(),
        1,
        reason,
        "restore the product invariant without adding a fallback",
        "cargo xtask policy --only product",
    )
}

#[derive(Default)]
struct LiteralVisitor {
    values: Vec<String>,
}
impl<'ast> Visit<'ast> for LiteralVisitor {
    fn visit_lit_str(&mut self, literal: &'ast LitStr) {
        self.values.push(literal.value());
    }
}

#[derive(Default)]
struct IdentifierVisitor {
    identifiers: BTreeSet<String>,
}
impl<'ast> Visit<'ast> for IdentifierVisitor {
    fn visit_path(&mut self, path: &'ast SynPath) {
        self.identifiers.extend(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string()),
        );
        syn::visit::visit_path(self, path);
    }
}

#[derive(Default)]
struct CloneVisitor {
    count: usize,
}
impl<'ast> Visit<'ast> for CloneVisitor {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if call.method == "clone" {
            self.count += 1;
        }
        syn::visit::visit_expr_method_call(self, call);
    }
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !cfg_test(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }
}

#[derive(Default)]
struct IngestLogVisitor {
    current: Option<String>,
    traced: bool,
    violations: Vec<String>,
}
impl<'ast> Visit<'ast> for IngestLogVisitor {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if !cfg_test(&function.attrs) {
            let name = function.sig.ident.to_string();
            self.enter(&name, |visitor| {
                syn::visit::visit_item_fn(visitor, function);
            });
        }
    }
    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        if !cfg_test(&function.attrs) {
            let name = function.sig.ident.to_string();
            self.enter(&name, |visitor| {
                syn::visit::visit_impl_item_fn(visitor, function);
            });
        }
    }
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !cfg_test(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }
    fn visit_macro(&mut self, mac: &'ast Macro) {
        if self.current.is_some()
            && mac
                .path
                .segments
                .first()
                .is_some_and(|segment| segment.ident == "tracing")
        {
            self.traced = true;
        }
        syn::visit::visit_macro(self, mac);
    }
}

impl IngestLogVisitor {
    fn enter(&mut self, name: &str, visit: impl FnOnce(&mut Self)) {
        let previous = self.current.replace(name.to_owned());
        let old_traced = self.traced;
        self.traced = false;
        visit(self);
        if name.starts_with("ingest") && self.traced {
            self.violations.push(name.to_owned());
        }
        self.current = previous;
        self.traced = old_traced;
    }
}

fn cfg_test(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        match &attribute.meta {
            Meta::List(list) => list
                .parse_args::<Meta>()
                .is_ok_and(|meta| matches!(meta, Meta::Path(path) if path.is_ident("test"))),
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests;
