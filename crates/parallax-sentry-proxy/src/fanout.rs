//! Independent destination workers with bounded delivery queues.

use std::collections::HashMap;

use bytes::Bytes;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestinationKind {
    Sentry,
    Rustrak,
    Parallax,
}

impl DestinationKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sentry => "sentry",
            Self::Rustrak => "rustrak",
            Self::Parallax => "parallax",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeliveryJob {
    pub destination: DestinationKind,
    pub url: String,
    pub auth: String,
    pub body: Bytes,
    pub content_type: &'static str,
}

#[derive(Debug)]
pub struct FanOut {
    senders: HashMap<DestinationKind, mpsc::Sender<DeliveryJob>>,
    _workers: Vec<JoinHandle<()>>,
}

impl FanOut {
    pub fn spawn(config: &Config) -> Self {
        let client = Client::new();
        let capacity = config.fanout.channel_capacity;
        let active = config.active_destinations();
        let mut senders = HashMap::new();
        let mut workers = Vec::new();

        for kind in &active {
            let (tx, rx) = mpsc::channel(capacity);
            let worker_client = client.clone();
            let worker_kind = *kind;
            workers.push(tokio::spawn(destination_worker(
                worker_kind,
                rx,
                worker_client,
            )));
            senders.insert(worker_kind, tx);
        }

        Self {
            senders,
            _workers: workers,
        }
    }

    /// Enqueue delivery jobs. Drops when a destination queue is full (scaffold).
    pub fn dispatch(&self, jobs: Vec<DeliveryJob>) {
        for job in jobs {
            let Some(sender) = self.senders.get(&job.destination) else {
                continue;
            };
            match sender.try_send(job) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!(destination = "full queue", "dropping delivery job");
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    warn!(destination = "worker closed", "dropping delivery job");
                }
            }
        }
    }
}

async fn destination_worker(
    kind: DestinationKind,
    mut rx: mpsc::Receiver<DeliveryJob>,
    client: Client,
) {
    while let Some(job) = rx.recv().await {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-sentry-auth"),
            HeaderValue::from_str(&job.auth).unwrap_or_else(|_| {
                HeaderValue::from_static("Sentry sentry_version=7, sentry_key=invalid")
            }),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(job.content_type));

        match client
            .post(&job.url)
            .headers(headers)
            .body(job.body.to_vec())
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    warn!(
                        destination = kind.as_str(),
                        status = %response.status(),
                        url = %job.url,
                        "destination rejected delivery"
                    );
                }
            }
            Err(error) => {
                warn!(
                    destination = kind.as_str(),
                    error = %error,
                    url = %job.url,
                    "destination delivery failed"
                );
            }
        }
    }
}
