//! Metric name identity across OTel dotted form and Greptime native tables.
//!
//! Native tables use Prometheus-style names (`catalog_product_queries_total`);
//! exemplar rows store the original OTel name (`catalog.product.queries`).
//! Lookups must treat those as one metric.

use super::native_metric_table_base;

/// Suffixes Greptime/Prometheus append to the native table base.
const NATIVE_METRIC_SUFFIXES: &[&str] = &[
    "_total",
    "_ratio",
    "_bytes",
    "_seconds",
    "_nanoseconds_total",
];

/// Every identifier that refers to the same metric as `name`.
#[must_use]
pub fn metric_name_aliases(name: &str) -> Vec<String> {
    let mut names = vec![name.to_string()];
    let native = native_metric_table_base(name);
    names.push(native.clone());
    for suffix in NATIVE_METRIC_SUFFIXES {
        if !native.ends_with(suffix) {
            names.push(format!("{native}{suffix}"));
        }
    }
    let mut stems = vec![name.to_string(), native];
    for suffix in NATIVE_METRIC_SUFFIXES {
        if let Some(stem) = name.strip_suffix(suffix) {
            stems.push(stem.to_string());
        }
    }
    for stem in stems {
        names.push(stem.clone());
        if stem.contains('_') && !stem.contains('.') {
            names.push(stem.replace('_', "."));
        }
    }
    names.sort();
    names.dedup();
    names
}

/// True when `requested` and `stored` name the same metric.
#[must_use]
pub fn metric_names_match(requested: &str, stored: &str) -> bool {
    metric_name_aliases(requested)
        .iter()
        .any(|alias| alias == stored)
}
