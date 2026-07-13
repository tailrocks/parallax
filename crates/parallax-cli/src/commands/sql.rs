//! Read-only telemetry SQL command.

use crate::client::{Client, gql_str};

/// `parallax sql "<SELECT …>"` — the engine's raw query power for agents
/// and ad-hoc digging; read-only, same guard as the API.
pub(crate) async fn sql(client: &Client, query: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ sql(query: "{}") {{ columns rows rowCount truncated }} }}"#,
            gql_str(query)
        ))
        .await?;
    let Some(result) = response.pointer("/data/sql").filter(|v| !v.is_null()) else {
        anyhow::bail!("sql query failed");
    };
    let columns: Vec<String> = result["columns"]
        .as_array()
        .map(|cols| {
            cols.iter()
                .filter_map(|c| c.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    println!("{}", columns.join("\t"));
    for row in result["rows"].as_array().into_iter().flatten() {
        let cells: Vec<String> = row
            .as_str()
            .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(s).ok())
            .map(|values| {
                values
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        println!("{}", cells.join("\t"));
    }
    let suffix = if result["truncated"].as_bool().unwrap_or(false) {
        " (truncated)"
    } else {
        ""
    };
    println!(
        "-- {} row(s){suffix}",
        result["rowCount"].as_i64().unwrap_or_default()
    );
    Ok(())
}
