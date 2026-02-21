// Shared API server functions

use dioxus::prelude::*;
use std::sync::OnceLock;

// HTTP client for forwarding requests to desktop app (server-only, not WASM)
#[cfg(not(target_arch = "wasm32"))]
static DESKTOP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn get_desktop_client() -> &'static reqwest::Client {
    DESKTOP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client for desktop app")
    })
}

// Helper function to forward request to desktop app and extract response
#[cfg(not(target_arch = "wasm32"))]
async fn forward_to_desktop_app(url: &str, data: &str) -> Result<String, ServerFnError> {
    let client = get_desktop_client();
    let json_body = serde_json::json!({"data": data});
    
    eprintln!("[Web Server] Forwarding request to {} with data: {}", url, data);
    println!("[Web Server] Forwarding request to {} with data: {}", url, data);
    
    match client.post(url)
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await {
        Ok(resp) => {
            let status = resp.status();
            eprintln!("[Web Server] Desktop app responded with status: {}", status);
            println!("[Web Server] Desktop app responded with status: {}", status);
            
            if status.is_success() {
                // Desktop app returns JSON with "message" field
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        eprintln!("[Web Server] Parsed JSON response: {:?}", json);
                        let message = json.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Success")
                            .to_string();
                        Ok(message)
                    }
                    Err(e) => {
                        eprintln!("[Web Server] Failed to parse JSON: {}", e);
                        Err(ServerFnError::new(format!("Failed to parse JSON response: {}", e)))
                    }
                }
            } else {
                // Get error text before response is consumed
                let error_text = match resp.text().await {
                    Ok(text) => {
                        eprintln!("[Web Server] Desktop app error response: {}", text);
                        text
                    },
                    Err(_) => format!("{}", status)
                };
                Err(ServerFnError::new(format!("Desktop app error: {}", error_text)))
            }
        }
        Err(e) => {
            let error_msg = format!("Desktop app not available: {}", e);
            eprintln!("[Web Server] Failed to forward to desktop app ({}): {}", url, e);
            eprintln!("[Web Server] Error details: {:?}", e);
            eprintln!("[Web Server] Error type - is_connect: {}, is_timeout: {}", e.is_connect(), e.is_timeout());
            println!("[Web Server] Failed to forward to desktop app ({}): {}", url, e);
            // Check if it's a connection error
            if e.is_connect() {
                eprintln!("[Web Server] Connection error - desktop app may not be running on port 8081");
                println!("[Web Server] Connection error - desktop app may not be running on port 8081");
            }
            if e.is_timeout() {
                eprintln!("[Web Server] Timeout error - desktop app may be slow to respond");
                println!("[Web Server] Timeout error - desktop app may be slow to respond");
            }
            Err(ServerFnError::new(error_msg))
        }
    }
}

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo_server(input: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Forward to desktop app's agent endpoint (server-only)
        let _ = forward_to_desktop_app("http://localhost:8081/api/agents/1/process", &input).await;
        // Always return input for echo, even if forwarding fails
        Ok(input)
    }
    #[cfg(target_arch = "wasm32")]
    {
        // WASM stub - this should never be called on client
        Ok(input)
    }
}

// ============================================================================
// HTTP/REST API Endpoints for Multi-Agent System
// ============================================================================

/// Process data through Agent 1 (forwards to desktop app)
#[post("/api/agents/1/process")]
pub async fn process_agent1(data: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("[Web Server] process_agent1 called with data: '{}' (len: {})", data, data.len());
        println!("[Web Server] process_agent1 called with data: '{}' (len: {})", data, data.len());
        
        if data.is_empty() {
            eprintln!("[Web Server] ERROR: data parameter is empty!");
            println!("[Web Server] ERROR: data parameter is empty!");
            return Err(ServerFnError::new("Data parameter is empty. Make sure to send 'data' field in form-urlencoded body."));
        }
        
        forward_to_desktop_app("http://localhost:8081/api/agents/1/process", &data).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

/// Process data through Agent 2 (forwards to desktop app)
#[post("/api/agents/2/process")]
pub async fn process_agent2(data: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        forward_to_desktop_app("http://localhost:8081/api/agents/2/process", &data).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

/// Process data through Agent 3 (forwards to desktop app)
#[post("/api/agents/3/process")]
pub async fn process_agent3(data: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        forward_to_desktop_app("http://localhost:8081/api/agents/3/process", &data).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

/// Process data through Agent 4 (forwards to desktop app)
#[post("/api/agents/4/process")]
pub async fn process_agent4(data: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        forward_to_desktop_app("http://localhost:8081/api/agents/4/process", &data).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

/// Process data through Agent 5 (forwards to desktop app)
#[post("/api/agents/5/process")]
pub async fn process_agent5(data: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        forward_to_desktop_app("http://localhost:8081/api/agents/5/process", &data).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

/// Get status of a specific agent (forwards to desktop app)
#[get("/api/agents/:id/status")]
pub async fn get_agent_status(id: u8) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = get_desktop_client();
        
        match client.get(&format!("http://localhost:8081/api/agents/{}/status", id))
            .send()
            .await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let message = json.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Success")
                                .to_string();
                            Ok(message)
                        }
                        Err(e) => {
                            Err(ServerFnError::new(format!("Failed to parse JSON response: {}", e)))
                        }
                    }
                } else {
                    let error_text = match resp.text().await {
                        Ok(text) => text,
                        Err(_) => format!("{}", status)
                    };
                    Err(ServerFnError::new(format!("Desktop app error: {}", error_text)))
                }
            }
            Err(e) => {
                eprintln!("[Web Server] Failed to forward status request to desktop app: {}", e);
                Err(ServerFnError::new(format!("Desktop app not available: {}", e)))
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

/// Process data through any agent (dynamic routing, forwards to desktop app)
#[post("/api/agents/:id/process")]
pub async fn process_agent_dynamic(id: u8, data: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        forward_to_desktop_app(&format!("http://localhost:8081/api/agents/{}/process", id), &data).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(ServerFnError::new("Agent endpoints not available in WASM"))
    }
}

