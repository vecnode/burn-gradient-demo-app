// Desktop-only HTTP server for agent endpoints
// Runs on port 8081 to handle agent requests from web server

#[cfg(feature = "desktop")]
use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
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

/// Start the desktop HTTP server for agent endpoints
/// Runs on port 8081
/// This function blocks forever, serving requests
#[cfg(feature = "desktop")]
pub async fn start_desktop_server() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure agents are initialized before starting server
    ensure_agents_initialized().await?;
    
    let app = Router::new()
        .route("/api/agents/:id/process", post(process_agent_handler))
        .route("/api/agents/:id/status", get(get_agent_status_handler))
        .layer(tower_http::cors::CorsLayer::permissive());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("[Desktop Server] Starting agent HTTP server on http://{}", addr);
    eprintln!("[Desktop Server] Starting agent HTTP server on http://{}", addr);
    
    // Write to log file
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        use std::io::Write;
        let _ = writeln!(file, "[Desktop Server] Starting agent HTTP server on http://{}", addr);
        let _ = file.flush();
    }
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("[Desktop Server] Server bound and ready to accept connections on http://{}", addr);
    eprintln!("[Desktop Server] Server bound and ready to accept connections on http://{}", addr);
    
    // Write to log file
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        use std::io::Write;
        let _ = writeln!(file, "[Desktop Server] Server bound and ready to accept connections");
        let _ = file.flush();
    }
    
    // This blocks forever, serving requests
    // If it returns, log the error
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[Desktop Server] Server error: {}", e);
        println!("[Desktop Server] Server error: {}", e);
        // Write to log file
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
            use std::io::Write;
            let _ = writeln!(file, "[Desktop Server] Server error: {}", e);
            let _ = file.flush();
        }
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Server error: {}", e))));
    }
    
    Ok(())
}

/// Handle agent process requests
#[cfg(feature = "desktop")]
async fn process_agent_handler(
    Path(id): Path<u8>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Log in Dioxus format - write to stderr (which tee captures) and log file
    // Use eprintln! which goes to stderr, and tee in serve-all.sh captures stderr
    let log_msg = format!("[Desktop Server] [200] POST /api/agents/{}/process", id);
    eprintln!("{}", log_msg);
    std::io::stderr().flush().ok();
    
    // Also write to log file for persistence
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        let _ = writeln!(file, "{}", log_msg);
        let _ = file.flush();
    }
    
    let data = payload.get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            eprintln!("[Desktop Server] Missing 'data' field in payload");
            std::io::stderr().flush().ok();
            StatusCode::BAD_REQUEST
        })?;
    
    if let Some(actor_ref) = get_agent(id) {
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: data.to_string(),
        });
        let response = json!({
            "status": "success",
            "message": format!("Message queued for Agent{}: {}", id, data)
        });
        Ok(Json(response))
    } else {
        eprintln!("[Desktop Server] Agent{} not found", id);
        std::io::stderr().flush().ok();
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handle agent status requests
#[cfg(feature = "desktop")]
async fn get_agent_status_handler(
    Path(id): Path<u8>,
) -> Result<Json<Value>, StatusCode> {
    // Log in Dioxus format - write to stderr (which tee captures) and log file
    // Use eprintln! which goes to stderr, and tee in serve-all.sh captures stderr
    let log_msg = format!("[Desktop Server] [200] GET /api/agents/{}/status", id);
    eprintln!("{}", log_msg);
    std::io::stderr().flush().ok();
    
    // Also write to log file for persistence
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
