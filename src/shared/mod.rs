// Shared components and server functions for desktop app
// Contains UI components and server functions that work together

pub mod api;

use dioxus::prelude::*;
use serde_json;

// Re-export API functions for convenience
pub use api::*;

/// System information component displaying CPU, GPU, and stack info
#[component]
pub fn SystemInfo() -> Element {
    // Use use_resource to fetch system info from server
    let mut retry_key = use_signal(|| 0u8);
    let mut system_info = use_resource(move || {
        let key = retry_key();
        async move {
            // Retry logic: wait longer and retry up to 3 times
            let max_retries = 3;
            let mut last_error = None;
            
            for attempt in 0..max_retries {
                // Increasing delay: 2s, 3s, 4s
                let delay_ms = 2000 + (attempt * 1000);
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                
                match get_system_info().await {
                    Ok(info) => return Ok(info),
                    Err(e) => {
                        let error_str = e.to_string();
                        // If it's a connection error and we have retries left, continue
                        if error_str.contains("Connection refused") || 
                           error_str.contains("Backend connection failed") {
                            if attempt < max_retries - 1 {
                                last_error = Some(e);
                                continue; // Retry
                            }
                        }
                        // Otherwise, return the error
                        return Err(e);
                    }
                }
            }
            
            // If we exhausted retries, return the last error or a default error
            match last_error {
                Some(e) => Err(e),
                None => Err(dioxus::prelude::ServerFnError::new("Failed to connect after multiple attempts"))
            }
        }
    });
    
    rsx! {
        div {
            id: "app-header",
            width: "50%",
            display: "flex",
            flex_direction: "row",
            gap: "4px",
            flex_wrap: "wrap",
            font_size: "10px",
            {
                match system_info() {
                    Some(Ok(info_str)) => {
                        if let Ok(info) = serde_json::from_str::<serde_json::Value>(&info_str) {
                            let cpu = info.get("cpu").and_then(|v| v.as_str()).unwrap_or("N/A");
                            let gpu = info.get("gpu").and_then(|v| v.as_str()).unwrap_or("N/A");
                            rsx! {
                                div { "CPU: {cpu}" }
                                div { "GPU: {gpu}" }
                            }
                        } else {
                            rsx! { div { "Error parsing system info" } }
                        }
                    }
                    Some(Err(e)) => {
                        let error_msg = e.to_string();
                        let is_connection_error = error_msg.contains("Connection refused") || 
                                                  error_msg.contains("Backend connection failed");
                        rsx! {
                            div {
                                color: "#ff6b6b",
                                font_size: "10px",
                                if is_connection_error {
                                    "Server connection not ready. Retrying automatically..."
                                } else {
                                    "Error: {error_msg}"
                                }
                            }
                            button {
                                font_size: "10px",
                                padding: "2px 8px",
                                margin_top: "5px",
                                onclick: move |_| {
                                    retry_key.set(retry_key() + 1);
                                    system_info.restart();
                                },
                                "Manual Retry"
                            }
                        }
                    }
                    None => {
                        rsx! { div { "Loading system info..." } }
                    }
                }
            }
        }
    }
}

/// Get system information (CPU, GPU, processor) using native Rust only
#[get("/api/system/info")]
pub async fn get_system_info() -> Result<String, ServerFnError> {
    use serde_json::json;
    use std::env;
    
    // Get CPU count from standard library (available since Rust 1.59)
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    
    // Get processor architecture
    let arch = env::consts::ARCH;
    let os = env::consts::OS;
    
    // Try to get CPU info from /proc/cpuinfo on Linux (native file reading)
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
    
    // Get GPU information from system files (native Rust, no extra dependencies)
    let gpu_info = if cfg!(target_os = "linux") {
        // Try to get GPU info from /sys/class/drm/ on Linux
        std::fs::read_dir("/sys/class/drm")
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .find_map(|entry| {
                        let path = entry.path();
                        let name = path.file_name()?.to_str()?;
                        // Look for card devices (not control nodes)
                        if name.starts_with("card") && !name.contains("-") {
                            // Try to read the device name
                            std::fs::read_to_string(path.join("device/uevent"))
                                .ok()
                                .and_then(|uevent| {
                                    uevent.lines()
                                        .find(|line| line.starts_with("DRIVER="))
                                        .map(|line| line.replace("DRIVER=", ""))
                                })
                                .or_else(|| {
                                    // Fallback: use the card name
                                    Some(name.to_string())
                                })
                        } else {
                            None
                        }
                    })
            })
            .map(|driver| format!("{} (via WGPU/Burn)", driver))
            .unwrap_or_else(|| "WGPU (Cross-platform GPU via Burn)".to_string())
    } else {
        // For non-Linux, we can't easily get GPU info without additional dependencies
        "WGPU (Cross-platform GPU via Burn)".to_string()
    };
    
    let info = json!({
        "cpu": format!("{} ({} cores)", cpu_name, cpu_count),
        "gpu": gpu_info,
    });
    
    Ok(info.to_string())
}
