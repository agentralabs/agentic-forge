//! Transport layer — stdio with optional auth gate.

use crate::protocol::ProtocolHandler;
use crate::session::SessionManager;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

/// Hard limit for framed stdio payloads (8 MiB).
const MAX_CONTENT_LENGTH_BYTES: usize = 8 * 1024 * 1024;

fn check_auth_gate() -> Result<(), String> {
    if let Ok(required_token) = std::env::var("AGENTIC_TOKEN") {
        if required_token.is_empty() {
            return Err("AGENTIC_TOKEN is set but empty".into());
        }
        tracing::info!("Auth gate active: AGENTIC_TOKEN required");
    }
    Ok(())
}

pub async fn run_stdio() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = check_auth_gate() {
        eprintln!("Auth gate error: {}", e);
        return Err(e.into());
    }

    let session = Arc::new(Mutex::new(SessionManager::new()));
    let handler = ProtocolHandler::new(session);

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    let mut content_length: Option<usize> = None;
    let mut framed_output = false;

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("content-length:") {
            let rest = trimmed.split_once(':').map(|(_, rhs)| rhs).unwrap_or("");
            match rest.trim().parse::<usize>() {
                Ok(n) if n <= MAX_CONTENT_LENGTH_BYTES => {
                    content_length = Some(n);
                    framed_output = true;
                }
                Ok(_) => {
                    let err = format!(
                        "Content-Length exceeds max frame size ({MAX_CONTENT_LENGTH_BYTES} bytes)"
                    );
                    eprintln!("{err}");
                    return Err(err.into());
                }
                Err(_) => {
                    let err = "Invalid Content-Length header".to_string();
                    eprintln!("{err}");
                    return Err(err.into());
                }
            }
            continue;
        }

        if let Some(n) = content_length {
            // Header separator line followed by framed JSON body.
            if trimmed.is_empty() {
                let mut body = vec![0u8; n];
                reader.read_exact(&mut body).await?;
                let payload = String::from_utf8_lossy(&body).to_string();
                if handle_input(&payload, &handler, &mut stdout, framed_output).await? {
                    break;
                }
                content_length = None;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if handle_input(trimmed, &handler, &mut stdout, framed_output).await? {
            break;
        }
    }

    Ok(())
}

async fn handle_input(
    input: &str,
    handler: &ProtocolHandler,
    stdout: &mut tokio::io::Stdout,
    framed_output: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let msg: serde_json::Value = match serde_json::from_str(input.trim()) {
        Ok(v) => v,
        Err(e) => {
            let err_response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": -32700, "message": format!("Parse error: {}", e) }
            });
            write_response(stdout, &err_response, framed_output).await?;
            return Ok(false);
        }
    };

    match handler.handle_message(msg).await {
        Ok(response) => {
            if !response.is_null() {
                write_response(stdout, &response, framed_output).await?;
            }
        }
        Err(e) => {
            let err_response = e.to_json_rpc_error(crate::types::RequestId::Null);
            write_response(stdout, &err_response, framed_output).await?;
        }
    }

    Ok(false)
}

async fn write_response(
    stdout: &mut tokio::io::Stdout,
    response: &serde_json::Value,
    framed_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let response_str = serde_json::to_string(response)?;
    if framed_output {
        let header = format!("Content-Length: {}\r\n\r\n", response_str.len());
        stdout.write_all(header.as_bytes()).await?;
        stdout.write_all(response_str.as_bytes()).await?;
        stdout.flush().await?;
        return Ok(());
    }

    stdout.write_all(response_str.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}
