//! Source-position and function-complexity measurements.

use super::*;

pub(super) fn line_at(source: &str, offset: u32) -> usize {
    source.as_bytes()[..offset as usize]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

pub(super) fn span_lines(source: &str, span: oxc_span::Span) -> (usize, usize) {
    let start = line_at(source, span.start);
    let end = line_at(source, span.end);
    (start, end.saturating_sub(start) + 1)
}

pub(super) fn function_health(
    source: &str,
    span: oxc_span::Span,
    complexity: (usize, usize),
) -> FunctionHealth {
    let (line, lines) = span_lines(source, span);
    FunctionHealth {
        line,
        lines,
        cyclomatic: complexity.0,
        cognitive: complexity.1,
    }
}

pub(super) fn branch(kind: AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::IfStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::SwitchCase(_)
            | AstKind::CatchClause(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::LogicalExpression(_)
    )
}
