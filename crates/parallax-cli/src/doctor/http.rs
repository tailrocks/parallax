//! Local HTTP probes for `parallax doctor`.

pub(super) async fn check_http(url: &str) -> Option<String> {
    check_http_body(url).await.map(|_| "ok".to_string())
}

pub(super) async fn check_http_body(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;
    let response = client.get(url).send().await.ok()?;
    response.status().is_success().then_some(())?;
    response.text().await.ok()
}

pub(super) async fn print_server_probes() -> anyhow::Result<()> {
    for (name, url) in [
        ("api (:4000)", "http://127.0.0.1:4000/health"),
        ("greptime child (:24000)", "http://127.0.0.1:24000/health"),
    ] {
        match check_http(url).await {
            Some(_) => println!("  {name}: ok"),
            None => println!("  {name}: NOT RESPONDING"),
        }
    }
    match check_http("http://127.0.0.1:4000/version").await {
        Some(_) => {
            let version = reqwest::get("http://127.0.0.1:4000/version")
                .await?
                .text()
                .await
                .unwrap_or_default();
            println!("  server version: {version}");
        }
        None => println!("  server version: unavailable (is `parallax serve` running?)"),
    }
    match check_http_body("http://127.0.0.1:4000/ingest/loss").await {
        Some(body) => println!("  ingest loss: {body}"),
        None => println!("  ingest loss: unavailable (is `parallax serve` running?)"),
    }
    Ok(())
}
