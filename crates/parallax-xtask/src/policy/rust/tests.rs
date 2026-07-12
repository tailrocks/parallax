use super::*;

#[test]
fn parses_functions_closures_and_comment_free_logical_lines() {
    let source = "// comment\nfn work() {\n  let f = || {\n    if true { 1 } else { 2 }\n  };\n}\n\n/* ignored */\n";
    let metric = analyze(source).expect("fixture should parse");
    assert_eq!(metric.logical_lines, 5);
    assert_eq!(metric.functions.len(), 2);
    assert_eq!(metric.functions[0].name, "work");
    assert!(metric.functions[1].cognitive > 0);
}

#[test]
fn malformed_rust_fails_closed() {
    assert!(analyze("fn {").is_err());
}
