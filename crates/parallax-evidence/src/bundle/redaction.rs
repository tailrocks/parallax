#[cfg(test)]
pub(crate) use parallax_redaction::redact;

pub(super) fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}
