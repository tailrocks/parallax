use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::cli::Output;
use crate::diagnostic::{Finding, Format, render};

const RERUN: &str = "cargo xtask docs links";

#[derive(Debug)]
struct Document {
    path: PathBuf,
    source: String,
    anchors: HashSet<String>,
}

pub(crate) fn run(root: &Path, output: Output) -> Result<()> {
    let paths = tracked_markdown(root)?;
    if paths.is_empty() {
        bail!("documentation link selection is empty");
    }
    let documents = load_documents(root, &paths)?;
    let known: HashMap<_, _> = documents
        .iter()
        .map(|document| (document.path.clone(), &document.anchors))
        .collect();
    let findings = documents
        .iter()
        .flat_map(|document| check_document(root, document, &known))
        .collect::<Vec<_>>();
    if !findings.is_empty() {
        print!("{}", render(&findings, output_format(output))?);
        bail!(
            "documentation link integrity found {} violation(s)",
            findings.len()
        );
    }
    println!(
        "documentation links passed ({} tracked Markdown files)",
        paths.len()
    );
    Ok(())
}

fn tracked_markdown(root: &Path) -> Result<Vec<PathBuf>> {
    let result = Command::new("git")
        .args(["ls-files", "-z", "--", "*.md", "**/*.md"])
        .current_dir(root)
        .output()
        .context("failed to list tracked Markdown files")?;
    if !result.status.success() {
        bail!("git ls-files failed with {}", result.status);
    }
    let mut paths = result
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| PathBuf::from(String::from_utf8_lossy(path).as_ref()))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn load_documents(root: &Path, paths: &[PathBuf]) -> Result<Vec<Document>> {
    paths
        .iter()
        .map(|path| {
            let source = std::fs::read_to_string(root.join(path))
                .with_context(|| format!("failed to read {}", path.display()))?;
            let anchors = heading_anchors(&source);
            Ok(Document {
                path: path.clone(),
                source,
                anchors,
            })
        })
        .collect()
}

fn heading_anchors(source: &str) -> HashSet<String> {
    let mut anchors = HashSet::new();
    let mut counts = HashMap::<String, usize>::new();
    let mut heading = None::<String>;
    for event in Parser::new_ext(source, markdown_options()) {
        match event {
            Event::Start(Tag::Heading { .. }) => heading = Some(String::new()),
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = &mut heading {
                    heading.push_str(&text);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(text) = heading.take() {
                    insert_heading_anchor(&mut anchors, &mut counts, &text);
                }
            }
            _ => {}
        }
    }
    anchors
}

fn insert_heading_anchor(
    anchors: &mut HashSet<String>,
    counts: &mut HashMap<String, usize>,
    heading: &str,
) {
    let base = github_slug(heading);
    let count = counts.entry(base.clone()).or_default();
    let anchor = match *count {
        0 => base,
        duplicate => format!("{base}-{duplicate}"),
    };
    *count += 1;
    anchors.insert(anchor);
}

fn check_document(
    root: &Path,
    document: &Document,
    known: &HashMap<PathBuf, &HashSet<String>>,
) -> Vec<Finding> {
    Parser::new_ext(&document.source, markdown_options())
        .into_offset_iter()
        .filter_map(|(event, range)| match event {
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                check_target(root, document, known, &dest_url, range.start)
            }
            _ => None,
        })
        .collect()
}

fn check_target(
    root: &Path,
    document: &Document,
    known: &HashMap<PathBuf, &HashSet<String>>,
    raw_target: &str,
    offset: usize,
) -> Option<Finding> {
    if raw_target.is_empty()
        || raw_target.starts_with('<')
        || raw_target.starts_with("http://")
        || raw_target.starts_with("https://")
        || raw_target.starts_with("mailto:")
        || raw_target.starts_with("data:")
        || raw_target.starts_with("app://")
    {
        return None;
    }
    let (raw_path, raw_fragment) = raw_target.split_once('#').unwrap_or((raw_target, ""));
    let decoded_path = match urlencoding::decode(raw_path) {
        Ok(path) => path,
        Err(error) => {
            return Some(finding(
                document,
                offset,
                raw_target,
                &format!("invalid percent encoding: {error}"),
            ));
        }
    };
    let source_dir = document.path.parent().unwrap_or_else(|| Path::new(""));
    let candidate = if decoded_path.is_empty() {
        document.path.clone()
    } else if decoded_path.starts_with('/') {
        PathBuf::from(decoded_path.trim_start_matches('/'))
    } else {
        source_dir.join(decoded_path.as_ref())
    };
    let Some(normalized) = normalize_repo_path(&candidate) else {
        return Some(finding(
            document,
            offset,
            raw_target,
            "target escapes repository root",
        ));
    };
    let absolute = root.join(&normalized);
    if !absolute.exists() {
        return Some(finding(
            document,
            offset,
            raw_target,
            "target does not exist",
        ));
    }
    if !raw_fragment.is_empty() && absolute.is_file() {
        let decoded_fragment =
            urlencoding::decode(raw_fragment).unwrap_or_else(|_| raw_fragment.into());
        let Some(anchors) = known.get(&normalized) else {
            return Some(finding(
                document,
                offset,
                raw_target,
                "fragment target is not tracked Markdown",
            ));
        };
        if !anchors.contains(decoded_fragment.as_ref()) {
            return Some(finding(
                document,
                offset,
                raw_target,
                "heading fragment does not exist",
            ));
        }
    }
    None
}

fn normalize_repo_path(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(normalized)
}

fn finding(document: &Document, offset: usize, target: &str, reason: &str) -> Finding {
    Finding::error(
        "docs.internal-link",
        &document.path.to_string_lossy(),
        1 + document.source[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count(),
        &format!("internal target `{target}` is invalid: {reason}"),
        "repair the target or its parsed heading fragment",
        RERUN,
    )
}

fn github_slug(heading: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in heading.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() || character == '_' || character == '-' {
            slug.push(character);
            previous_dash = false;
        } else if character.is_whitespace() && !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_owned()
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_GFM
}

fn output_format(output: Output) -> Format {
    match output {
        Output::Human => Format::Human,
        Output::Json => Format::Json,
        Output::Github => Format::Github,
    }
}

#[cfg(test)]
mod tests;
