
use super::*;

#[test]
fn local_stdio_requires_explicit_cli_opt_in() {
    let default = Cli::try_parse_from(["parallax-mcp-spike"]).expect("parse default");
    let trusted =
        Cli::try_parse_from(["parallax-mcp-spike", "--allow-local-stdio"]).expect("parse opt-in");

    assert!(!default.allow_local_stdio);
    assert!(trusted.allow_local_stdio);
}

#[test]
fn api_url_is_loopback_only_until_remote_auth_lands() {
    for accepted in [
        "http://127.0.0.1:4000/",
        "http://127.42.0.9:4000",
        "http://[::1]:4000",
    ] {
        assert!(
            gql::normalize_local_base_url(accepted).is_ok(),
            "{accepted}"
        );
    }
    for denied in [
        "https://localhost:4000",
        "http://localhost:4000",
        "http://example.com:4000",
        "http://user:secret@localhost:4000",
        "http://localhost:4000/graphql",
        "http://localhost:4000?token=secret",
        "not-a-url",
    ] {
        assert!(gql::normalize_local_base_url(denied).is_err(), "{denied}");
    }
}
