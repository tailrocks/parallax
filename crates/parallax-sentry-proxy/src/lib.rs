//! Sentry-compatible envelope fan-out proxy (Selene §11 scaffold).
//!
//! Accepts SDK traffic on a stable proxy DSN, validates envelope framing via
//! [`parallax_ingest::parse_envelope`], rewrites transport auth/DSN metadata
//! per destination, and delivers through independent bounded worker queues.
//! Durability, circuit breaking, and rate-limit handling are follow-ups.

pub mod config;
pub mod dsn;
pub mod fanout;
pub mod http;

pub use config::Config;
pub use fanout::FanOut;
