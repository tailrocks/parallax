//! CLI for plan 090 read-transport measurement.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use read_transport_bench::{
    dataset_counts, run_seed, HttpClient, HttpFormat, MysqlClient, INVENTORY, METRIC_SERIES_RANGE_SQL,
    PARTITION_QUERIES,
};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "read-transport-bench", about = "Plan 090 GreptimeDB read transport spike")]
struct Cli {
    #[arg(
        long,
        env = "GREPTIME_HTTP",
        default_value = "http://127.0.0.1:24000"
    )]
    http: String,

    #[arg(long, env = "GREPTIME_MYSQL", default_value = "mysql://127.0.0.1:24002/public")]
    mysql: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Seed {
        #[arg(long, default_value_t = 100_000)]
        n: u64,
    },
    Bench {
        #[arg(long, default_value_t = 50)]
        reps: usize,
        #[arg(long, default_value_t = 5)]
        warmup: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    PartitionBench {
        #[arg(long, default_value_t = 50)]
        reps: usize,
        #[arg(long, default_value_t = 5)]
        warmup: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    RangeCheck {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let http = HttpClient::new(&cli.http)?;

    match cli.cmd {
        Command::Seed { n } => {
            run_seed(&http, n).await?;
            let counts = dataset_counts(&http).await?;
            let version = http.engine_version().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "engine_version": version,
                    "n": n,
                    "counts": counts,
                }))?
            );
        }
        Command::Status => {
            let version = http.engine_version().await.unwrap_or_else(|e| format!("ERR:{e}"));
            let counts = dataset_counts(&http).await.unwrap_or(json!({"error": true}));
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "engine_version": version,
                    "http": cli.http,
                    "mysql": cli.mysql,
                    "counts": counts,
                }))?
            );
        }
        Command::Bench { reps, warmup, out } => {
            let version = http.engine_version().await?;
            let counts = dataset_counts(&http).await?;
            let formats = [
                HttpFormat::GreptimeV1,
                HttpFormat::Arrow,
                HttpFormat::ArrowZstd,
            ];
            let mut results = Vec::new();
            for q in INVENTORY {
                for fmt in formats {
                    eprintln!("bench {} / {} …", q.id, fmt.label());
                    let stats = http
                        .measure_http(fmt, q, reps, warmup)
                        .await
                        .with_context(|| format!("{} / {}", q.id, fmt.label()))?;
                    results.push(stats);
                }
            }

            let mut mysql_results = Vec::new();
            let mut reconnect_ms = None;
            match MysqlClient::new(&cli.mysql) {
                Ok(mysql) => {
                    for q in INVENTORY {
                        eprintln!("bench {} / mysql_prepared …", q.id);
                        match mysql.measure_prepared(q, reps, warmup).await {
                            Ok((stats, reconnect)) => {
                                reconnect_ms = Some(reconnect.as_secs_f64() * 1000.0);
                                mysql_results.push(stats);
                            }
                            Err(e) => {
                                eprintln!("WARN mysql_prepared {}: {e:#}", q.id);
                            }
                        }
                    }
                    let _ = mysql.disconnect().await;
                }
                Err(e) => eprintln!("WARN mysql client: {e:#}"),
            }

            let report = json!({
                "engine_version": version,
                "dataset": counts,
                "reps": reps,
                "warmup": warmup,
                "http": results,
                "mysql_prepared": mysql_results,
                "mysql_pool_reconnect_ms": reconnect_ms,
            });
            let text = serde_json::to_string_pretty(&report)?;
            if let Some(path) = out {
                std::fs::write(&path, &text)?;
                eprintln!("wrote {}", path.display());
            }
            println!("{text}");
        }
        Command::PartitionBench { reps, warmup, out } => {
            let version = http.engine_version().await?;
            let counts = dataset_counts(&http).await?;
            let mut results = Vec::new();
            for q in PARTITION_QUERIES {
                eprintln!("partition-bench {} …", q.id);
                let stats = http
                    .measure_http(HttpFormat::GreptimeV1, q, reps, warmup)
                    .await
                    .with_context(|| q.id.to_string())?;
                results.push(stats);
            }
            let report = json!({
                "engine_version": version,
                "dataset": counts,
                "reps": reps,
                "note": "traces_p16 uses 4 SQL RANGE partitions as a multi-region proxy; native OTLP default is 16 hash partitions via trace_table_partitions hint",
                "results": results,
            });
            let text = serde_json::to_string_pretty(&report)?;
            if let Some(path) = out {
                std::fs::write(&path, &text)?;
            }
            println!("{text}");
        }
        Command::RangeCheck { out } => {
            let date_bin = INVENTORY
                .iter()
                .find(|q| q.id == "metric_series")
                .expect("metric_series in inventory")
                .sql;
            let explain_date = http
                .sql_json(&format!("EXPLAIN ANALYZE {date_bin}"))
                .await?;
            let explain_range = http
                .sql_json(&format!("EXPLAIN ANALYZE {METRIC_SERIES_RANGE_SQL}"))
                .await?;
            let mut db_times = Vec::new();
            let mut rg_times = Vec::new();
            for _ in 0..20 {
                let s = http
                    .run_http(HttpFormat::GreptimeV1, date_bin)
                    .await?;
                db_times.push(s.wall_ms);
                let s = http
                    .run_http(HttpFormat::GreptimeV1, METRIC_SERIES_RANGE_SQL)
                    .await?;
                rg_times.push(s.wall_ms);
            }
            db_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            rg_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let report = json!({
                "date_bin_sql": date_bin,
                "range_sql": METRIC_SERIES_RANGE_SQL,
                "explain_date_bin": explain_date,
                "explain_range": explain_range,
                "date_bin_p50_ms": db_times[db_times.len()/2],
                "range_p50_ms": rg_times[rg_times.len()/2],
                "date_bin_row_count_sample": http.run_http(HttpFormat::GreptimeV1, date_bin).await?.rows,
                "range_row_count_sample": http.run_http(HttpFormat::GreptimeV1, METRIC_SERIES_RANGE_SQL).await?.rows,
            });
            let text = serde_json::to_string_pretty(&report)?;
            if let Some(path) = out {
                std::fs::write(&path, &text)?;
            }
            println!("{text}");
        }
    }
    Ok(())
}
