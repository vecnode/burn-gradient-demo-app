// Desktop application component

#[cfg(feature = "desktop")]
use dioxus::prelude::*;
#[cfg(feature = "desktop")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(feature = "desktop")]
use std::time::Duration;
#[cfg(feature = "desktop")]
use burn::backend::{Autodiff, wgpu::Wgpu};
#[cfg(feature = "desktop")]
use std::sync::OnceLock;
#[cfg(feature = "desktop")]
use std::io::Write;

#[cfg(feature = "desktop")]
use crate::shared::{SystemInfo, echo_server};
#[cfg(feature = "desktop")]
use crate::agents::ensure_agents_initialized;
#[cfg(feature = "desktop")]
use crate::app::desktop_server;

// Reusable HTTP client for all desktop-to-web communication
// Reduces connection overhead and improves reliability
#[cfg(feature = "desktop")]
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

#[cfg(feature = "desktop")]
fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client")
    })
}

// Global cognitive cycle state
#[cfg(feature = "desktop")]
static COGNITIVE_CYCLE_STATE: AtomicBool = AtomicBool::new(false);
// Global cognitive cycle counter
#[cfg(feature = "desktop")]
static COGNITIVE_CYCLE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "desktop")]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(feature = "desktop")]
const MAIN_CSS: Asset = asset!("/assets/main.css");

/// Desktop application root component
#[cfg(feature = "desktop")]
#[component]
pub fn DesktopApp() -> Element {
    let mut cycle_state = use_signal(|| COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst));
    
    // Initialize agents and start desktop HTTP server on startup
    use_effect(move || {
        spawn(async move {
            // Small delay to ensure desktop app is fully initialized
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            // Initialize agents in desktop app
            if let Err(e) = ensure_agents_initialized().await {
                eprintln!("[Desktop] Failed to initialize agents: {}", e);
            } else {
                println!("[Desktop] Agents initialized successfully");
                eprintln!("[Desktop] Agents initialized successfully");
            }
            
            // Start desktop HTTP server for agent endpoints (runs in background)
            spawn(async move {
                // Small delay before starting server
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Err(e) = desktop_server::start_desktop_server().await {
                    eprintln!("[Desktop Server] Error: {}", e);
                    println!("[Desktop Server] Error: {}", e);
                }
            });
        });
    });
    
    use_effect(move || {
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            loop {
                interval.tick().await;
                if COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst) {
                    COGNITIVE_CYCLE_COUNTER.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
    });
    
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        DesktopHeader {}
        br {}
        SystemInfo {}
        br {}
        div {
            id: "app-header",
            width: "40%",
            button {
                onclick: move |_| {
                    let new_state = !COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst);
                    COGNITIVE_CYCLE_STATE.store(new_state, Ordering::SeqCst);
                    cycle_state.set(new_state);
                    println!("cognitive_cycle_state={}", new_state);
                },
                if cycle_state() { "Stop CogCycle" } else { "Start CogCycle" }
            }
            div {
                width: "20px",
                height: "20px",
                background_color: if cycle_state() { "#006400" } else { "#8B0000" },
                margin_left: "10px",
            }
        }
        br {}
        div {
            id: "app-header",
            width: "40%",
            button {
                onclick: move |_| {
                    // Write to log file that serve-all.sh monitors (most reliable)
                    let msg = "[Desktop] ========================================\n[Desktop] Building LSTM model\n[Desktop] ========================================";
                    println!("{}", msg);
                    eprintln!("{}", msg);
                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                        use std::io::Write;
                        let _ = writeln!(file, "{}", msg);
                        let _ = file.flush();
                    }
                    
                    spawn(async move {
                        type Backend = Autodiff<Wgpu>;
                        let device = Default::default();
                        let config = crate::lstm::LstmConfig::default();
                        let lstm = crate::lstm::Lstm::<Backend>::new(config, &device);
                        let result_msg = format!("[Desktop] LSTM model built successfully:\n{:#?}", lstm);
                        println!("{}", result_msg);
                        eprintln!("{}", result_msg);
                        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                            use std::io::Write;
                            let _ = writeln!(file, "{}", result_msg);
                            let _ = file.flush();
                        }
                    });
                },
                "Build LSTM"
            }
            button {
                onclick: move |_| {
                    // Write to log file that serve-all.sh monitors (most reliable)
                    let msg = "[Desktop] ========================================\n[Desktop] Computing tensor gradients\n[Desktop] ========================================";
                    println!("{}", msg);
                    eprintln!("{}", msg);
                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                        use std::io::Write;
                        let _ = writeln!(file, "{}", msg);
                        let _ = file.flush();
                    }
                    
                    spawn(async move {
                        crate::burn_tensor_example();
                        let result_msg = "[Desktop] Tensor gradients computed successfully";
                        println!("{}", result_msg);
                        eprintln!("{}", result_msg);
                        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                            use std::io::Write;
                            let _ = writeln!(file, "{}", result_msg);
                            let _ = file.flush();
                        }
                    });
                },
                "Compute Tensor Gradients"
            }
        }
        DesktopEcho {}
        br {}
        div {
            id: "app-header",
            width: "40%",
            p {
                font_size: "12px",
                "MCP Server Test"
            }
        }
        DesktopMCP {}
        br {}
        div {
            id: "app-header",
            width: "40%",
            p {
                font_size: "12px",
                "HTTP Communication Test - Send to Web Browser"
            }
            TestButton {}
        }
    }
}

#[cfg(feature = "desktop")]
#[component]
fn TestButton() -> Element {
    let mut status_msg = use_signal(|| String::new());
    
    rsx! {
        button {
            onclick: move |_| {
                let mut status = status_msg;
                
                spawn(async move {
                    let test_msg = "Hello from desktop app - Test message!";
                    
                    // Use shared HTTP client for efficient connection reuse
                    let client = get_http_client();
                    let json_body = serde_json::json!({"message": test_msg});
                    
                    match client.post("http://localhost:8080/api/messages/send")
                        .header("Content-Type", "application/json")
                        .json(&json_body)
                        .send()
                        .await {
                        Ok(resp) => {
                            let http_status = resp.status();
                            if http_status.is_success() {
                                status.set("✓ Sent!".to_string());
                            } else {
                                let error_msg = resp.text().await.unwrap_or_else(|_| format!("{}", http_status));
                                status.set(format!("✗ Error: {} - {}", http_status, error_msg));
                            }
                        }
                        Err(e) => {
                            status.set(format!("✗ Failed: {}", e));
                        }
                    }
                });
            },
            "Send Test Message to Browser"
        }
        if !status_msg().is_empty() {
            p {
                color: if status_msg().starts_with("✓") { "#4caf50" } else { "#f44336" },
                "{status_msg}"
            }
        }
    }
}

#[cfg(feature = "desktop")]
#[component]
fn DesktopHeader() -> Element {
    rsx! {
        div {
            id: "app-header",
            "pattern-clock - Desktop"
        }
    }
}

/// Echo component that demonstrates fullstack server functions (Desktop)
#[cfg(feature = "desktop")]
#[component]
fn DesktopEcho() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            h5 { "ServerFn Echo" }
            br {}
            input {
                placeholder: "Type here to echo.",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// MCP Server component for testing MCP tools
/// Desktop app calls MCP tools directly and prints to terminal
#[cfg(feature = "desktop")]
#[component]
fn DesktopMCP() -> Element {
    let mut is_loading = use_signal(|| false);

    rsx! {
        div {
            button {
                disabled: is_loading(),
                onclick: move |_| {
                    is_loading.set(true);
                    spawn(async move {
                        // Call MCP tool directly and print to terminal
                        let mcp_server = crate::mcp_server::PatternClockMCP::new();
                        let result = mcp_server.call_example_tool().await;
                        println!("[MCP] Example Tool Result: {}", result);
                        is_loading.set(false);
                    });
                },
                if is_loading() { "Loading..." } else { "Call MCP Example Tool" }
            }
        }
        div {
            button {
                disabled: is_loading(),
                onclick: move |_| {
                    is_loading.set(true);
                    spawn(async move {
                        // Call MCP tool directly and print to terminal
                        let mcp_server = crate::mcp_server::PatternClockMCP::new();
                        let result = mcp_server.call_get_random_number().await;
                        println!("[MCP] Random Number Result: {}", result);
                        is_loading.set(false);
                    });
                },
                if is_loading() { "Loading..." } else { "Get MCP Random Number" }
            }
        }
    }
}
