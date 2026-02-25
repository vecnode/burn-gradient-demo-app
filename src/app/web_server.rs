// Web server module - serves WebApp on port 8080 from within desktop app
// This allows the desktop app to embed the web interface in an iframe

#[cfg(feature = "desktop")]
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
#[cfg(feature = "desktop")]
use serde_json::{json, Value};
#[cfg(feature = "desktop")]
use std::net::SocketAddr;
#[cfg(feature = "desktop")]
use std::io::Write;
#[cfg(feature = "desktop")]
use crate::agents::{get_agent, ensure_agents_initialized, AgentMessage};
#[cfg(feature = "desktop")]
use std::collections::VecDeque;
#[cfg(feature = "desktop")]
use std::sync::Mutex;
#[cfg(feature = "desktop")]
use std::sync::OnceLock;

/// Start the web server on port 8080
/// Serves the WebApp and handles all API endpoints
/// This runs in the same process as the desktop app
#[cfg(feature = "desktop")]
pub async fn start_web_server() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure agents are initialized
    ensure_agents_initialized().await?;
    
    let app = Router::new()
        // API endpoints - call agents directly (same process, no forwarding needed)
        .route("/api/agents/1/process", post(process_agent1_handler))
        .route("/api/agents/2/process", post(process_agent2_handler))
        .route("/api/agents/3/process", post(process_agent3_handler))
        .route("/api/agents/4/process", post(process_agent4_handler))
        .route("/api/agents/5/process", post(process_agent5_handler))
        .route("/api/agents/:id/status", get(get_agent_status_handler))
        .route("/api/echo", post(echo_handler))
        .route("/api/messages/send", post(send_message_handler))
        // Return 404 silently for /api/messages/stream - not used, prevents flood from cached WASM/browser tabs
        .route("/api/messages/stream", get(|| async { 
            // Return 404 without logging to reduce terminal spam
            StatusCode::NOT_FOUND 
        }))
        .route("/api/system/info", get(get_system_info_handler))
        // Serve WebApp - for now return a simple HTML page with iframe
        // In production, this would serve the compiled WASM
        .route("/", get(serve_webapp))
        .layer(tower_http::cors::CorsLayer::permissive());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("[Web Server] Starting web server on http://{}", addr);
    eprintln!("[Web Server] Starting web server on http://{}", addr);
    
    // Write to log file
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        let _ = writeln!(file, "[Web Server] Starting web server on http://{}", addr);
        let _ = file.flush();
    }
    
    // Try to bind to port - if it's already in use, Dioxus server is likely running
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            // Port already in use - Dioxus fullstack server is running on 8080
            // This is expected, so we'll skip starting our server
            eprintln!("[Web Server] Port 8080 already in use - Dioxus server is running, skipping our web server");
            println!("[Web Server] Port 8080 already in use - Dioxus server is running, skipping our web server");
            return Ok(()); // Exit gracefully
        }
        Err(e) => return Err(Box::new(e)),
    };
    
    println!("[Web Server] Server bound and ready to accept connections on http://{}", addr);
    eprintln!("[Web Server] Server bound and ready to accept connections on http://{}", addr);
    
    // Write to log file
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        let _ = writeln!(file, "[Web Server] Server bound and ready to accept connections");
        let _ = file.flush();
    }
    
    // This blocks forever, serving requests
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[Web Server] Server error: {}", e);
        println!("[Web Server] Server error: {}", e);
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
            let _ = writeln!(file, "[Web Server] Server error: {}", e);
            let _ = file.flush();
        }
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Server error: {}", e))));
    }
    
    Ok(())
}

/// Message buffer for reliable message delivery (same as shared/communication.rs)
#[cfg(feature = "desktop")]
static MESSAGE_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

#[cfg(feature = "desktop")]
fn get_message_buffer() -> &'static Mutex<VecDeque<String>> {
    MESSAGE_BUFFER.get_or_init(|| Mutex::new(VecDeque::with_capacity(1000)))
}

/// Serve the WebApp HTML page
/// In a full implementation, this would serve the compiled WASM bundle
#[cfg(feature = "desktop")]
async fn serve_webapp() -> Html<String> {
    // Return a simple HTML page that loads the WebApp
    // In production, this would serve the compiled Dioxus WASM bundle
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>pattern-clock - Web Interface</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <div id="main"></div>
    <script>
        // Placeholder - in production, this would load the Dioxus WASM bundle
        document.getElementById('main').innerHTML = '<h1>pattern-clock Web Interface</h1><p>Web interface will be served here. For now, use the embedded iframe in the desktop app.</p>';
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

/// Handle agent 1 process requests - direct call (same process)
#[cfg(feature = "desktop")]
async fn process_agent1_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    process_agent_handler(1, payload).await
}

/// Handle agent 2 process requests - direct call (same process)
#[cfg(feature = "desktop")]
async fn process_agent2_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    process_agent_handler(2, payload).await
}

/// Handle agent 3 process requests - direct call (same process)
#[cfg(feature = "desktop")]
async fn process_agent3_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    process_agent_handler(3, payload).await
}

/// Handle agent 4 process requests - direct call (same process)
#[cfg(feature = "desktop")]
async fn process_agent4_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    process_agent_handler(4, payload).await
}

/// Handle agent 5 process requests - direct call (same process)
#[cfg(feature = "desktop")]
async fn process_agent5_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    process_agent_handler(5, payload).await
}

/// Common handler for agent process requests
#[cfg(feature = "desktop")]
async fn process_agent_handler(id: u8, payload: Value) -> Result<Json<Value>, StatusCode> {
    let log_msg = format!("[Web Server] [200] POST /api/agents/{}/process", id);
    eprintln!("{}", log_msg);
    std::io::stderr().flush().ok();
    
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        let _ = writeln!(file, "{}", log_msg);
        let _ = file.flush();
    }
    
    let data = payload.get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            eprintln!("[Web Server] Missing 'data' field in payload");
            std::io::stderr().flush().ok();
            StatusCode::BAD_REQUEST
        })?;
    
    if let Some(actor_ref) = get_agent(id) {
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: data.to_string(),
        });
        Ok(Json(json!({
            "status": "success",
            "message": format!("Message queued for Agent{}: {}", id, data)
        })))
    } else {
        eprintln!("[Web Server] Agent{} not found", id);
        std::io::stderr().flush().ok();
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handle agent status requests - direct call (same process)
#[cfg(feature = "desktop")]
async fn get_agent_status_handler(Path(id): Path<u8>) -> Result<Json<Value>, StatusCode> {
    let log_msg = format!("[Web Server] [200] GET /api/agents/{}/status", id);
    eprintln!("{}", log_msg);
    std::io::stderr().flush().ok();
    
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        let _ = writeln!(file, "{}", log_msg);
        let _ = file.flush();
    }
    
    if let Some(actor_ref) = get_agent(id) {
        let _ = actor_ref.send_message(AgentMessage::GetStatus);
        Ok(Json(json!({
            "status": "success",
            "message": format!("Status request sent to Agent{}", id)
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handle echo requests (accepts both JSON and form data)
#[cfg(feature = "desktop")]
async fn echo_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    let input = payload.get("input")
        .or_else(|| payload.get("data"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // Forward to agent 1 (same process, direct call)
    if let Some(actor_ref) = get_agent(1) {
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: input.to_string(),
        });
    }
    
    Ok(Json(json!({"result": input})))
}

/// Handle send message requests
#[cfg(feature = "desktop")]
async fn send_message_handler(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    let message = payload.get("message")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Store in buffer directly (same process)
    let mut buffer = get_message_buffer().lock().unwrap();
    buffer.push_back(message.to_string());
    if buffer.len() > 1000 {
        buffer.pop_front();
    }
    
    Ok(Json(json!({"status": "success"})))
}

/// Handle system info requests
#[cfg(feature = "desktop")]
async fn get_system_info_handler() -> Result<Json<Value>, StatusCode> {
    use serde_json::json;
    use std::env;
    
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    
    let arch = env::consts::ARCH;
    let os = env::consts::OS;
    
    let cpu_name = if cfg!(target_os = "linux") {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("model name"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| format!("{} ({})", arch, os))
    } else {
        format!("{} ({})", arch, os)
    };
    
    let gpu_info = if cfg!(target_os = "linux") {
        std::fs::read_dir("/sys/class/drm")
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .find_map(|entry| {
                        let path = entry.path();
                        let name = path.file_name()?.to_str()?;
                        if name.starts_with("card") && !name.contains("-") {
                            std::fs::read_to_string(path.join("device/uevent"))
                                .ok()
                                .and_then(|uevent| {
                                    uevent.lines()
                                        .find(|line| line.starts_with("DRIVER="))
                                        .map(|line| line.replace("DRIVER=", ""))
                                })
                                .or_else(|| Some(name.to_string()))
                        } else {
                            None
                        }
                    })
            })
            .map(|driver| format!("{} (via WGPU/Burn)", driver))
            .unwrap_or_else(|| "WGPU (Cross-platform GPU via Burn)".to_string())
    } else {
        "WGPU (Cross-platform GPU via Burn)".to_string()
    };
    
    Ok(Json(json!({
        "cpu": format!("{} ({} cores)", cpu_name, cpu_count),
        "gpu": gpu_info,
    })))
}
