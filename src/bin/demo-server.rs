use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = req.get("id").cloned() else {
            continue;
        };
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let resp = match handle(method, &req) {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, msg)) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": msg } })
            }
        };
        if writeln!(out, "{resp}").is_err() || out.flush().is_err() {
            break;
        }
    }
}

fn handle(method: &str, req: &Value) -> Result<Value, (i64, String)> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": req["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "demo-remote", "version": "0.1.0" }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": [
            {
                "name": "echo",
                "description": "Gönderilen mesajı aynen geri döndürür",
                "inputSchema": {
                    "type": "object",
                    "properties": { "message": { "type": "string" } },
                    "required": ["message"]
                }
            },
            {
                "name": "get_time",
                "description": "Uzak PC'nin geçerli saatini ISO formatında döndürür",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "add",
                "description": "İki sayıyı toplar",
                "inputSchema": {
                    "type": "object",
                    "properties": { "a": { "type": "number" }, "b": { "type": "number" } },
                    "required": ["a", "b"]
                }
            }
        ] })),
        "tools/call" => {
            let name = req["params"]["name"].as_str().unwrap_or("");
            eprintln!("[demo] tool cagrildi: {name}");
            let args = &req["params"]["arguments"];
            let text = match name {
                "echo" => args["message"].as_str().unwrap_or_default().to_string(),
                "get_time" => chrono::Utc::now().to_rfc3339(),
                "add" => {
                    let a = args["a"].as_f64().unwrap_or_default();
                    let b = args["b"].as_f64().unwrap_or_default();
                    format!("{}", a + b)
                }
                _ => {
                    return Ok(json!({
                        "isError": true,
                        "content": [{ "type": "text", "text": format!("Bilinmeyen tool: {name}") }]
                    }));
                }
            };
            Ok(json!({ "content": [{ "type": "text", "text": text }] }))
        }
        _ => Err((-32601, format!("Method not found: {method}"))),
    }
}
