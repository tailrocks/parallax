//! GraphQL depth/complexity pre-checks (plan 024 heritage).

use std::collections::BTreeMap;

use crate::{MAX_ROWS, Schema};

/// Clamp a GraphQL `limit` argument to `[0, MAX_ROWS]`.
///
/// `None` uses `default` (then still capped at `MAX_ROWS`). Negative and
/// zero values stay at 0 — they do **not** fall back to `default`.
pub(crate) fn clamp_limit(limit: Option<i32>, default: usize) -> usize {
    limit
        .map_or(default, |l| usize::try_from(l.max(0)).unwrap_or(default))
        .min(MAX_ROWS)
}

#[derive(Debug, Default)]
struct QueryShape {
    max_depth: usize,
    field_count: usize,
}

type ParsedSelection<'a> = juniper::Selection<'a, juniper::DefaultScalarValue>;

fn walk_selections<'a>(
    selections: &[ParsedSelection<'a>],
    fragments: &BTreeMap<&'a str, Vec<ParsedSelection<'a>>>,
    depth: usize,
    stats: &mut QueryShape,
    fragment_stack: &mut Vec<&'a str>,
) -> Result<(), String> {
    for selection in selections {
        match selection {
            juniper::Selection::Field(field) => {
                let field_depth = depth + 1;
                stats.max_depth = stats.max_depth.max(field_depth);
                stats.field_count += 1;
                if let Some(children) = &field.item.selection_set {
                    walk_selections(children, fragments, field_depth, stats, fragment_stack)?;
                }
            }
            juniper::Selection::InlineFragment(fragment) => {
                walk_selections(
                    &fragment.item.selection_set,
                    fragments,
                    depth,
                    stats,
                    fragment_stack,
                )?;
            }
            juniper::Selection::FragmentSpread(spread) => {
                let name = spread.item.name.item;
                if fragment_stack.contains(&name) {
                    return Err(format!("GraphQL fragment cycle includes `{name}`"));
                }
                if let Some(fragment) = fragments.get(name) {
                    fragment_stack.push(name);
                    walk_selections(fragment, fragments, depth, stats, fragment_stack)?;
                    fragment_stack.pop();
                }
            }
        }
    }
    Ok(())
}

/// Enforce coarse query-cost ceilings before Juniper execution.
///
/// Juniper 0.17 has no built-in depth/complexity middleware. Depth is selected
/// field nesting; complexity is approximated as total selected fields,
/// including fragment expansions.
pub fn check_query_limits(
    schema: &Schema,
    query: &str,
    operation_name: Option<&str>,
    max_depth: usize,
    max_complexity: usize,
) -> Result<(), String> {
    let document = juniper::parser::parse_document_source::<juniper::DefaultScalarValue>(
        query,
        &schema.schema,
    )
    .map_err(|error| format!("GraphQL query parse failed: {error}"))?;
    let fragments: BTreeMap<_, Vec<_>> = document
        .iter()
        .filter_map(|definition| match definition {
            juniper::Definition::Fragment(fragment) => {
                Some((fragment.item.name.item, fragment.item.selection_set.clone()))
            }
            _ => None,
        })
        .collect();

    let mut stats = QueryShape::default();
    let mut matched_operation = false;
    for definition in &document {
        let juniper::Definition::Operation(operation) = definition else {
            continue;
        };
        let op_name = operation.item.name.as_ref().map(|name| name.item);
        if operation_name.is_some_and(|wanted| op_name != Some(wanted)) {
            continue;
        }
        matched_operation = true;
        walk_selections(
            &operation.item.selection_set,
            &fragments,
            0,
            &mut stats,
            &mut Vec::new(),
        )?;
    }

    if !matched_operation && operation_name.is_some() {
        return Ok(());
    }
    if stats.max_depth > max_depth {
        return Err(format!(
            "GraphQL query depth {} exceeds configured maximum {}",
            stats.max_depth, max_depth
        ));
    }
    if stats.field_count > max_complexity {
        return Err(format!(
            "GraphQL query field count {} exceeds configured maximum {}",
            stats.field_count, max_complexity
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::build_schema;

    #[test]
    fn clamp_limit_boundaries() {
        assert_eq!(clamp_limit(None, 25), 25);
        assert_eq!(clamp_limit(Some(-5), 25), 0);
        assert_eq!(clamp_limit(Some(0), 25), 0);
        assert_eq!(clamp_limit(Some(i32::MAX), 25), MAX_ROWS);
        assert_eq!(clamp_limit(None, MAX_ROWS + 10), MAX_ROWS);
    }

    #[test]
    fn check_query_limits_named_operation_and_fragment_cycle() {
        let schema = build_schema();
        check_query_limits(&schema, "query { __typename }", None, 8, 1_000).expect("simple query");
        check_query_limits(
            &schema,
            "query One { __typename } query Two { __typename }",
            Some("Two"),
            8,
            1_000,
        )
        .expect("named operation");
        let cycled = r"
            fragment A on Query { ...B }
            fragment B on Query { ...A }
            query { ...A }
        ";
        let err = check_query_limits(&schema, cycled, None, 8, 1_000).expect_err("cycle");
        assert!(err.contains("fragment cycle"), "{err}");
    }
}
