use super::*;

#[expect(clippy::expect_used, reason = "static regex literal")]
pub(super) fn static_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex")
}

pub(super) fn redaction_rules() -> &'static [(&'static str, Regex, &'static str)] {
    static CELL: OnceLock<Vec<(&'static str, Regex, &'static str)>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            (
                "dsn_userinfo",
                static_regex(r"://[^/\s:@]+:[^/\s@]+@"),
                "://[REDACTED:dsn_userinfo]@",
            ),
            (
                "private_key_block",
                static_regex(
                    r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
                ),
                "[REDACTED:private_key_block]",
            ),
            (
                "github_token",
                static_regex(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b"),
                "[REDACTED:github_token]",
            ),
            (
                "github_pat",
                static_regex(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
                "[REDACTED:github_pat]",
            ),
            (
                "slack_token",
                static_regex(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),
                "[REDACTED:slack_token]",
            ),
            (
                "jwt",
                static_regex(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:jwt]",
            ),
            (
                "aws_access_key_id",
                static_regex(r"\bAKIA[0-9A-Z]{16}\b"),
                "[REDACTED:aws_access_key_id]",
            ),
            (
                "aws_secret_access_key",
                static_regex(r"(?i)\baws[_.-]?secret[_.-]?access[_.-]?key\b\s*[=:]\s*\S+"),
                "[REDACTED:aws_secret_access_key]",
            ),
            (
                "bearer_token",
                static_regex(r"Bearer\s+[A-Za-z0-9._\-]{8,}"),
                "[REDACTED:bearer_token]",
            ),
            (
                "stripe_live_key",
                static_regex(r"\bsk_live_[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:stripe_live_key]",
            ),
            (
                "stripe_test_key",
                static_regex(r"\bsk_test_[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:stripe_test_key]",
            ),
            (
                "anthropic_api_key",
                static_regex(r"\bsk-ant-[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:anthropic_api_key]",
            ),
            (
                "openai_api_key",
                static_regex(r"\bsk-[A-Za-z0-9]{20,}\b"),
                "[REDACTED:openai_api_key]",
            ),
            (
                "google_api_key",
                static_regex(r"\bAIza[0-9A-Za-z_-]{10,}\b"),
                "[REDACTED:google_api_key]",
            ),
            (
                "gitlab_pat",
                static_regex(r"\bglpat-[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:gitlab_pat]",
            ),
            (
                "npm_token",
                static_regex(r"\bnpm_[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:npm_token]",
            ),
            (
                "basic_auth",
                // Padding ends in a non-word character, so a trailing word
                // boundary would miss ordinary Base64 credentials.
                static_regex(r"(?i)\bBasic\s+[A-Za-z0-9+/]{8,}={0,2}"),
                "[REDACTED:basic_auth]",
            ),
            (
                "password_assignment",
                static_regex(r"(?i)password\s*[=:]\s*\S+"),
                "[REDACTED:password_assignment]",
            ),
            (
                "generic_secret_assignment",
                // Exclude bare `auth` (collides with `auth=Bearer …` after the
                // bearer rule rewrites the token). Values must not start with
                // `[` so already-redacted markers are not re-matched.
                static_regex(
                    r#"(?i)\b(?:api[_-]?key|apikey|secret|token|passwd|pwd|access[_-]?key)\b\s*[=:]\s*[^\s"'\[\]]{6,}"#,
                ),
                "[REDACTED:generic_secret_assignment]",
            ),
            (
                "email_address",
                static_regex(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b"),
                "[REDACTED:email_address]",
            ),
        ]
    })
}

pub(crate) fn redact(text: &str, report: &mut RedactionReport) -> String {
    let mut out = text.to_string();
    for (name, rule, replacement) in redaction_rules() {
        let hits = rule.find_iter(&out).count() as u64;
        if hits > 0 {
            out = rule.replace_all(&out, *replacement).into_owned();
            *report.redacted_counts.entry(name).or_insert(0) += hits;
        }
    }
    let control_characters = out
        .chars()
        .filter(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        .count() as u64;
    if control_characters > 0 {
        out.retain(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'));
        *report
            .redacted_counts
            .entry("control_character")
            .or_insert(0) += control_characters;
    }
    out
}

pub(super) fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_authorization_with_base64_padding_is_redacted() {
        let mut report = RedactionReport {
            policy: "test",
            ..Default::default()
        };
        let output = redact("Authorization: Basic dXNlcjpwYXNzd29yZHh4eHg=", &mut report);
        assert_eq!(output, "Authorization: [REDACTED:basic_auth]");
        assert_eq!(report.redacted_counts.get("basic_auth"), Some(&1));
    }

    #[test]
    fn terminal_controls_are_removed_but_text_whitespace_is_preserved() {
        let mut report = RedactionReport {
            policy: "test",
            ..Default::default()
        };
        let output = redact("safe\n\t\u{1b}[31mred\u{0}\u{7f}\rtext", &mut report);

        assert_eq!(output, "safe\n\t[31mred\rtext");
        assert_eq!(report.redacted_counts.get("control_character"), Some(&3));
    }
}
