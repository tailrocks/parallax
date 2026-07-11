//! Shared GraphQL wrapper types used across domains.

use juniper::graphql_object;
use parallax_storage::model::SeriesPoint;

use crate::{ApiContext, nanos_string};

pub struct Point(pub(crate) SeriesPoint);

#[graphql_object(context = ApiContext)]
impl Point {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn value(&self) -> f64 {
        self.0.value
    }
}
