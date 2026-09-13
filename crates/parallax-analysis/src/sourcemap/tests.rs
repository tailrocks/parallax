use super::*;

/// Hand-computed v3 map (see module docs for the VLQ derivation):
/// `src/app.ts`, names `onClick`/`render`, generated line 1 with three
/// segments: col 0 → 1:0, col 10 → 1:4 (`onClick`), col 25 → 3:2 (`render`).
fn fixture_map() -> String {
    serde_json::json!({
        "version": 3,
        "file": "app.min.js",
        "sources": ["src/app.ts"],
        "names": ["onClick", "render"],
        "mappings": "AAAA,UAAIA,eAEFC"
    })
    .to_string()
}

#[test]
fn decodes_hand_computed_segments() {
    let map = parse_source_map(&fixture_map()).expect("fixture parses");
    assert_eq!(map.sources(), &["src/app.ts".to_string()]);
    assert_eq!(
        map.resolve(1, 0),
        Some(ResolvedPosition {
            source: "src/app.ts".to_string(),
            line: 1,
            column: 0,
            name: None,
        })
    );
    assert_eq!(
        map.resolve(1, 10),
        Some(ResolvedPosition {
            source: "src/app.ts".to_string(),
            line: 1,
            column: 4,
            name: Some("onClick".to_string()),
        })
    );
    assert_eq!(
        map.resolve(1, 25),
        Some(ResolvedPosition {
            source: "src/app.ts".to_string(),
            line: 3,
            column: 2,
            name: Some("render".to_string()),
        })
    );
}

#[test]
fn resolves_greatest_lower_bound_between_segments() {
    let map = parse_source_map(&fixture_map()).expect("fixture parses");
    // Column 12 sits between the col-10 and col-25 starts: binds to col 10.
    let resolved = map.resolve(1, 12).expect("between segments");
    assert_eq!(resolved.column, 4);
    assert_eq!(resolved.name.as_deref(), Some("onClick"));
}

#[test]
fn unmapped_positions_resolve_to_none() {
    let map = parse_source_map(&fixture_map()).expect("fixture parses");
    assert_eq!(map.resolve(0, 0), None, "line numbers are 1-based");
    assert_eq!(map.resolve(2, 0), None, "generated line 2 has no segments");
}

#[test]
fn unmapped_boundary_segment_resolves_to_none() {
    // `A` = a 1-field (unmapped) segment at column 0; col 0 must not resolve
    // but must still act as a column floor for later segments on the line.
    let raw = serde_json::json!({
        "version": 3,
        "sources": ["a.js"],
        "names": [],
        "mappings": "A,CAAC"
    })
    .to_string();
    let map = parse_source_map(&raw).expect("parses");
    assert_eq!(map.resolve(1, 0), None);
    let resolved = map.resolve(1, 1).expect("second segment resolves");
    assert_eq!(resolved.source, "a.js");
    assert_eq!((resolved.line, resolved.column), (1, 1));
}

#[test]
fn rejects_non_v3_and_malformed_maps() {
    for (name, json) in [
        ("not json", "{nope"),
        ("missing version", r#"{"sources":["a"],"mappings":"AAAA"}"#),
        (
            "wrong version",
            r#"{"version":2,"sources":["a"],"mappings":"AAAA"}"#,
        ),
        (
            "no sources",
            r#"{"version":3,"sources":[],"mappings":"AAAA"}"#,
        ),
        ("no mappings", r#"{"version":3,"sources":["a"]}"#),
        (
            "bad vlq",
            r#"{"version":3,"sources":["a"],"mappings":"AAAA,!!!"}"#,
        ),
        (
            "truncated vlq",
            r#"{"version":3,"sources":["a"],"mappings":"g"}"#,
        ),
        (
            "bad field count",
            r#"{"version":3,"sources":["a"],"mappings":"AAA"}"#,
        ),
    ] {
        assert!(parse_source_map(json).is_err(), "{name} must be rejected");
    }
}

#[test]
fn parses_v8_frames_and_skips_other_languages() {
    let stack = "Error: boom\n    at onClick (https://cdn.example.com/app.min.js:1:25)\n    at https://cdn.example.com/app.min.js:1:10\n    at render (app.min.js:2:5)\nFile \"x.py\", line 3, in f\n    at com.Example.run(Example.java:42)\nplain text";
    let frames = parse_js_frames(stack);
    assert_eq!(frames.len(), 3);
    assert_eq!(
        frames[0],
        ParsedFrame {
            raw: "at onClick (https://cdn.example.com/app.min.js:1:25)".to_string(),
            function: Some("onClick".to_string()),
            file: "https://cdn.example.com/app.min.js".to_string(),
            line: 1,
            column: 25,
        }
    );
    assert_eq!(frames[1].function, None);
    assert_eq!(frames[1].line, 1);
    assert_eq!(frames[2].file, "app.min.js");
}

#[test]
fn frame_file_key_strips_url_wrapping() {
    assert_eq!(
        frame_file_key("https://cdn.example.com/s/app.min.js?debug=1#frag"),
        "app.min.js"
    );
    assert_eq!(frame_file_key("app.min.js"), "app.min.js");
    assert_eq!(frame_file_key("/static/js/app.min.js"), "app.min.js");
}
