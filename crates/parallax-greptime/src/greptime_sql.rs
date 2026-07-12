use parallax_proto::semconv;

pub(crate) fn escape(text: &str) -> String {
    text.replace('\'', "''")
}

pub(crate) fn escape_ident(text: &str) -> String {
    text.replace('"', "\"\"")
}

pub(crate) fn quoted_ident(text: &str) -> String {
    format!(r#""{}""#, escape_ident(text))
}

pub(crate) fn resource_attr_ident(attribute: &str) -> String {
    quoted_ident(&semconv::resource_column(attribute))
}

pub(crate) fn wire_attr_ident(attribute: &str) -> String {
    quoted_ident(attribute)
}

pub(crate) fn resource_json_get(attribute: &str) -> String {
    format!(
        r#"json_get_string("resource_attributes", '{}')"#,
        semconv::resource_json_path(attribute)
    )
}

pub(crate) fn log_service_name_expr() -> String {
    format!(
        r#"COALESCE("service.name", {})"#,
        resource_json_get(semconv::SERVICE_NAME)
    )
}

// Greptime metric-engine point tables discovered with live DESCRIBE expose
// `greptime_timestamp` and `greptime_value`; explicit histogram bucket tables
// add `le`. They are bookkeeping, not groupable metric labels.
pub(crate) const METRIC_BOOKKEEPING_COLUMNS: &[&str] =
    &["greptime_timestamp", "greptime_value", "le"];

fn native_metric_base(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn metric_table_candidates(name: &str, suffix: Option<&str>) -> Vec<String> {
    let suffix = suffix.unwrap_or_default();
    let mut bases = vec![name.to_string()];
    let native = native_metric_base(name);
    if native != name {
        bases.push(native.clone());
    }
    if !native.ends_with("_total") {
        bases.push(format!("{native}_total"));
    }
    for unit_suffix in ["_ratio", "_bytes", "_seconds", "_nanoseconds_total"] {
        if !native.ends_with(unit_suffix) {
            bases.push(format!("{native}{unit_suffix}"));
        }
    }

    let mut candidates = Vec::new();
    for base in bases {
        let candidate = format!("{base}{suffix}");
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

pub(crate) fn runtime_display_name(base: &str) -> Option<String> {
    const PREFIXES: &[(&str, &str)] = &[
        ("process_", "process."),
        ("system_", "system."),
        ("jvm_", "jvm."),
        ("container_", "container."),
        ("db_client_connection_", "db.client.connection."),
    ];
    if let Some(rest) = base.strip_prefix("tokio_runtime_") {
        return Some(format!("tokio.runtime.{rest}"));
    }
    PREFIXES.iter().find_map(|(native, display)| {
        base.strip_prefix(native)
            .map(|rest| format!("{display}{}", rest.replace('_', ".")))
    })
}

const METRIC_DISPLAY_ALIASES: &[(&str, &str)] = &[
    ("tokio.runtime.alive.tasks", "tokio.runtime.alive_tasks"),
    (
        "tokio.runtime.blocking.pool.depth",
        "tokio.runtime.blocking_pool_depth",
    ),
    (
        "tokio.runtime.global.queue.depth",
        "tokio.runtime.global_queue_depth",
    ),
    (
        "tokio.runtime.total.busy.duration.ms",
        "tokio.runtime.total_busy_duration_ms",
    ),
    (
        "tokio.runtime.total.park.count",
        "tokio.runtime.total_park_count",
    ),
    ("tokio.runtime.workers.count", "tokio.runtime.workers_count"),
    ("process.cpu.utilization.ratio", "process.cpu.utilization"),
    ("process.memory.usage.bytes", "process.memory.usage"),
];

pub(crate) fn canonical_metric_display_name(name: &str) -> String {
    METRIC_DISPLAY_ALIASES
        .iter()
        .find_map(|(legacy, canonical)| (*legacy == name).then_some((*canonical).to_string()))
        .unwrap_or_else(|| name.to_string())
}

fn metric_name_query_names(name: &str) -> Vec<String> {
    let mut names = vec![name.to_string(), canonical_metric_display_name(name)];
    for (legacy, canonical) in METRIC_DISPLAY_ALIASES {
        if *canonical == name {
            names.push((*legacy).to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

pub(crate) fn metric_name_sql_filter(column: &str, name: &str) -> String {
    let names = metric_name_query_names(name);
    if names.len() == 1 {
        format!(r#"{column} = '{}'"#, escape(&names[0]))
    } else {
        let quoted = names
            .iter()
            .map(|name| format!("'{}'", escape(name)))
            .collect::<Vec<_>>()
            .join(",");
        format!("{column} IN ({quoted})")
    }
}
