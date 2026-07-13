use super::*;
use tempfile::tempdir;

#[test]
fn parser_handles_anchors_and_ignores_fenced_links() -> Result<()> {
    let anchors = heading_anchors("# Hello, 世界!\n\n# Hello, 世界!\n");
    let document = Document {
        path: PathBuf::from("README.md"),
        source: "```md\n[broken](missing.md)\n```\n".into(),
        anchors: HashSet::new(),
    };
    let actual = (
        anchors.contains("hello-世界"),
        anchors.contains("hello-世界-1"),
        check_document(Path::new("."), &document, &HashMap::new()).is_empty(),
        normalize_repo_path(Path::new("docs/../README.md")),
        normalize_repo_path(Path::new("../../outside.md")),
        valid_target_fixture(),
        invalid_target_fixture(),
    );
    let expected = (
        true,
        true,
        true,
        Some(PathBuf::from("README.md")),
        None,
        true,
        (3, vec![1, 2, 3]),
    );
    if actual != expected {
        bail!("Markdown link fixture mismatch: {actual:?}");
    }
    Ok(())
}

fn valid_target_fixture() -> bool {
    let root = tempdir().unwrap_or_else(|error| panic!("temporary root: {error}"));
    std::fs::create_dir(root.path().join("assets"))
        .unwrap_or_else(|error| panic!("asset directory: {error}"));
    std::fs::write(root.path().join("assets/a b.png"), b"image")
        .unwrap_or_else(|error| panic!("image fixture: {error}"));
    std::fs::write(root.path().join("target.md"), "# Target Heading\n")
        .unwrap_or_else(|error| panic!("target fixture: {error}"));
    let source = concat!(
        "[inline](target.md#target-heading)\n",
        "[reference][target]\n",
        "![image](assets/a%20b.png)\n",
        "[directory](assets/)\n",
        "[target]: target.md#target-heading\n",
    );
    let document = Document {
        path: PathBuf::from("README.md"),
        source: source.into(),
        anchors: HashSet::new(),
    };
    let target_anchors = heading_anchors("# Target Heading\n");
    let known = HashMap::from([(PathBuf::from("target.md"), &target_anchors)]);
    check_document(root.path(), &document, &known).is_empty()
}

fn invalid_target_fixture() -> (usize, Vec<usize>) {
    let root = tempdir().unwrap_or_else(|error| panic!("temporary root: {error}"));
    std::fs::write(root.path().join("target.md"), "# Existing\n")
        .unwrap_or_else(|error| panic!("target fixture: {error}"));
    let document = Document {
        path: PathBuf::from("docs/source.md"),
        source: concat!(
            "[fragment](../target.md#missing)\n",
            "[file](missing.md)\n",
            "[escape](../../outside.md)\n",
        )
        .into(),
        anchors: HashSet::new(),
    };
    let target_anchors = heading_anchors("# Existing\n");
    let known = HashMap::from([(PathBuf::from("target.md"), &target_anchors)]);
    let findings = check_document(root.path(), &document, &known);
    (
        findings.len(),
        findings.iter().map(|finding| finding.line).collect(),
    )
}
