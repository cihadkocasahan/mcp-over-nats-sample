use async_nats::ConnectOptions;
use futures::StreamExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into());
    let subject = std::env::var("MCP_SUBJECT").unwrap_or_else(|_| "mcp.demo".into());

    let mut opts = ConnectOptions::new();
    if let Ok(path) = std::env::var("NATS_CREDS") {
        opts = opts.credentials(&std::fs::read_to_string(&path)?)?;
    }
    let client = opts.connect(&url).await?;
    eprintln!("[proxy] NATS: {url} | subject: {subject}");

    let inbox = client.new_inbox();
    let mut sub = client.subscribe(inbox.clone()).await?;
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(msg) = sub.next().await {
            if stdout.write_all(&msg.payload).await.is_err() {
                break;
            }
            let _ = stdout.flush().await;
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                let id = v
                    .get("id")
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "-".into());
                eprintln!("[proxy] <- NATS yanit | id={id}");
            }
        }
    });

    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("?");
            let id = v
                .get("id")
                .map(|i| i.to_string())
                .unwrap_or_else(|| "-".into());
            let tool = v["params"]["name"].as_str().unwrap_or("");
            eprintln!("[proxy] -> NATS {subject} | id={id} method={method} {tool}");
        }
        client
            .publish_with_reply(subject.clone(), inbox.clone(), line.into())
            .await?;
    }
    client.flush().await?;
    Ok(())
}
