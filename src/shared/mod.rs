// Shared components and utilities used by both desktop and web platforms

pub mod api;
pub mod communication;

use dioxus::prelude::*;
use serde_json;

// Re-export API functions for convenience
pub use api::*;
// Re-export communication functions
pub use communication::{send_message, stream_messages};

/// System information component displaying CPU, GPU, and stack info
#[component]
pub fn SystemInfo() -> Element {
    let mut system_info = use_resource(move || async move {
        get_system_info().await.unwrap_or_else(|_| "{}".to_string())
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
                    Some(info_str) => {
                        if let Ok(info) = serde_json::from_str::<serde_json::Value>(&info_str) {
                            let cpu = info.get("cpu").and_then(|v| v.as_str()).unwrap_or("N/A");
                            let gpu = info.get("gpu").and_then(|v| v.as_str()).unwrap_or("N/A");
                            rsx! {
                                div { "CPU: {cpu}" }
                                div { "GPU: {gpu}" }
                            }
                        } else {
                            rsx! { div { "Loading system info" } }
                        }
                    }
                    None => {
                        rsx! { div { "Loading system info" } }
                    }
                }
            }
        }
    }
}

/// Helper function to convert markdown to HTML
pub fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, Options, html};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
