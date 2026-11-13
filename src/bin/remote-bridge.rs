use async_nats::ConnectOptions;
use futures::StreamExt;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into());
    let subject = std::env::var("MCP_SUBJECT").unwrap_or_else(|_| "mcp.demo".into());
    let server_bin = std::env::var("MCP_SERVER_BIN").unwrap_or_else(|_| {
        let exe = std::env::current_exe().expect("current_exe");
        exe.parent()
            .expect("exe parent")
            .join(format!("demo-server{}", std::env::consts::EXE_SUFFIX))
            .to_string_lossy()
            .into_owned()
    });

    let mut opts = ConnectOptions::new();
    if let Ok(path) = std::env::var("NATS_CREDS") {
        opts = opts.credentials(&std::fs::read_to_string(&path)?)?;
    }
    let client = opts.connect(&url).await?;
    eprintln!("[remote] NATS: {url} | subject: {subject} | server: {server_bin}");

    let mut sub = client.subscribe(subject.clone()).await?;
    let mut sessions: HashMap<String, mpsc::Sender<Vec<u8>>> = HashMap::new();

    while let Some(msg) = sub.next().await {
        let Some(reply) = msg.reply else { continue };
        let reply = reply.to_string();
        if !sessions.contains_key(&reply) {
            let mut child = Command::new(&server_bin)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .kill_on_drop(true)
                .spawn()?;
            let mut child_in = child.stdin.take().expect("child stdin");
            let child_out = BufReader::new(child.stdout.take().expect("child stdout"));
            let (tx, mut rx) = mpsc::channel::<Vec<u8>>(64);

            tokio::spawn(async move {
                let _child = child;
                while let Some(data) = rx.recv().await {
                    if child_in.write_all(&data).await.is_err() {
                        break;
                    }
                    let _ = child_in.flush().await;
                }
            });

            let c = client.clone();
            let r = reply.clone();
            tokio::spawn(async move {
                let mut lines = child_out.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                        let id = v
                            .get("id")
                            .map(|i| i.to_string())
                            .unwrap_or_else(|| "-".into());
                        eprintln!("[remote] <- child | id={id} -> inbox");
                    }
                    if c.publish(r.clone(), (line + "\n").into()).await.is_err() {
                        break;
                    }
                }
            });

            sessions.insert(reply.clone(), tx);
            eprintln!("[remote] session başladı: {reply}");
        }
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
            let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("?");
            let id = v
                .get("id")
                .map(|i| i.to_string())
                .unwrap_or_else(|| "-".into());
            let tool = v["params"]["name"].as_str().unwrap_or("");
            eprintln!("[remote] -> child | id={id} method={method} {tool}");
        }
        let mut data = msg.payload.to_vec();
        data.push(b'\n');
        if sessions[&reply].send(data).await.is_err() {
            sessions.remove(&reply);
        }
    }
    Ok(())
}
