// Desktop application component

#[cfg(feature = "desktop")]
use dioxus::prelude::*;
#[cfg(feature = "desktop")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "desktop")]
use burn::backend::{Autodiff, wgpu::Wgpu};

#[cfg(feature = "desktop")]
use crate::shared::{SystemInfo, echo_server};
#[cfg(feature = "desktop")]
use crate::agents::ensure_agents_initialized;
#[cfg(feature = "desktop")]
use crate::app::desktop_server;

// Global cognitive cycle state
#[cfg(feature = "desktop")]
static COGNITIVE_CYCLE_STATE: AtomicBool = AtomicBool::new(false);
// Guard to prevent duplicate initialization
#[cfg(feature = "desktop")]
static INITIALIZATION_STARTED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "desktop")]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(feature = "desktop")]
const MAIN_CSS: Asset = asset!("/assets/main.css");

/// Desktop application root component
#[cfg(feature = "desktop")]
#[component]
pub fn DesktopApp() -> Element {
    let mut cycle_state = use_signal(|| COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst));
    
    // Initialize agents and start desktop HTTP server on startup (only once)
    use_effect(move || {
        spawn(async move {
            // Use atomic flag INSIDE the spawned task to ensure initialization only happens once
            // This prevents race conditions where multiple use_effect calls spawn tasks before the flag is set
            if INITIALIZATION_STARTED.swap(true, Ordering::SeqCst) {
                // Already initialized, skip
                return;
            }
            
            // Small delay to ensure desktop app is fully initialized
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            // Initialize agents in desktop app
            if let Err(e) = ensure_agents_initialized().await {
                eprintln!("[Desktop] Failed to initialize agents: {}", e);
            } else {
                eprintln!("[Desktop] Agents initialized successfully");
            }
            
            // Start desktop HTTP server for agent endpoints on port 8081 (runs in background)
            spawn(async move {
                // Small delay before starting server
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                if let Err(e) = desktop_server::start_desktop_server().await {
                    eprintln!("[Desktop Server] Error: {}", e);
                }
            });
        });
    });
    
    let mut active_tab = use_signal(|| 0u8);
    
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        
        // Top bar with tabs
        TabBar {
            active_tab,
            tabs: vec![
                ("Overview", 0),
                ("System", 1),
                ("ML Operations", 2),
                ("Echo Test", 3),
            ],
        }
        
        // Tab content
        div {
            padding: "20px",
            if active_tab() == 0 {
                OverviewTab { cycle_state }
            }
            if active_tab() == 1 {
                SystemTab {}
            }
            if active_tab() == 2 {
                MLOperationsTab {}
            }
            if active_tab() == 3 {
                DesktopEcho {}
            }
        }
    }
}

/// Tab bar component with clickable tabs
#[cfg(feature = "desktop")]
#[component]
fn TabBar(active_tab: Signal<u8>, tabs: Vec<(&'static str, u8)>) -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            border_bottom: "1px solid #333",
            background_color: "#2a2a2a",
            padding: "0px",
            gap: "0px",
            for (label, tab_id) in tabs {
                button {
                    padding: "6px 12px",
                    border: "none",
                    background_color: if active_tab() == tab_id { "#3a3a3a" } else { "transparent" },
                    border_bottom: if active_tab() == tab_id { "1px solid #007bff" } else { "1px solid transparent" },
                    cursor: "pointer",
                    font_size: "12px",
                    color: if active_tab() == tab_id { "#fff" } else { "#aaa" },
                    onclick: move |_| active_tab.set(tab_id),
                    {label}
                }
            }
        }
    }
}

/// Overview tab - shows cognitive cycle controls
#[cfg(feature = "desktop")]
#[component]
fn OverviewTab(cycle_state: Signal<bool>) -> Element {
    rsx! {
        div {
            div { font_size: "14px", "Overview" }
            br {}
            div {
                display: "flex",
                flex_direction: "row",
                align_items: "center",
                gap: "10px",
                button {
                    onclick: move |_| {
                        let new_state = !COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst);
                        COGNITIVE_CYCLE_STATE.store(new_state, Ordering::SeqCst);
                        cycle_state.set(new_state);
                        let msg = format!("[Desktop] Cognitive Cycle {}: cognitive_cycle_state={}", 
                            if new_state { "STARTED" } else { "STOPPED" }, new_state);
                        eprintln!("{}", msg);
                        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                            use std::io::Write;
                            let _ = writeln!(file, "{}", msg);
                            let _ = file.flush();
                        }
                    },
                    if cycle_state() { "Stop CogCycle" } else { "Start CogCycle" }
                }
                div {
                    width: "20px",
                    height: "20px",
                    background_color: if cycle_state() { "#006400" } else { "#8B0000" },
                    border_radius: "50%",
                }
            }
        }
    }
}

/// System tab - shows system information
#[cfg(feature = "desktop")]
#[component]
fn SystemTab() -> Element {
    rsx! {
        div {
            div { font_size: "12px", "System Information" }
            br {}
            SystemInfo {}
        }
    }
}

/// ML Operations tab - shows ML-related buttons
#[cfg(feature = "desktop")]
#[component]
fn MLOperationsTab() -> Element {
    let mut directory_path = use_signal(|| {
        // Expand ~ to home directory
        std::env::var("HOME")
            .map(|home| format!("{}/Desktop", home))
            .unwrap_or_else(|_| "~/Desktop".to_string())
    });
    
    // Dataset storage - holds paths to loaded files
    let mut dataset_paths = use_signal(|| Vec::<String>::new());
    
    rsx! {
        div {
            div { font_size: "12px", "ML Operations" }
            br {}
            div {
                display: "flex",
                flex_direction: "column",
                gap: "10px",
                width: "fit-content",
                // Directory picker row
                div {
                    display: "flex",
                    flex_direction: "row",
                    gap: "10px",
                    align_items: "center",
                    input {
                        width: "300px",
                        value: "{directory_path}",
                        oninput: move |e| directory_path.set(e.value()),
                    }
                    button {
                        width: "fit-content",
                        onclick: move |_| {
                            spawn(async move {
                                // Use rfd (Rust File Dialog) for cross-platform file dialogs
                                use rfd::FileDialog;
                                
                                // Expand ~ in path if present for initial directory
                                let initial_path = directory_path().replace("~", &std::env::var("HOME").unwrap_or_else(|_| String::new()));
                                let initial_dir = std::path::Path::new(&initial_path);
                                
                                let dialog = if initial_dir.exists() {
                                    FileDialog::new().set_directory(initial_dir)
                                } else {
                                    FileDialog::new()
                                };
                                
                                if let Some(path) = dialog.pick_folder() {
                                    let path_str = path.to_string_lossy().to_string();
                                    directory_path.set(path_str.clone());
                                    
                                    let msg = format!("[Desktop] Selected directory: {}", path_str);
                                    eprintln!("{}", msg);
                                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                                        use std::io::Write;
                                        let _ = writeln!(file, "{}", msg);
                                        let _ = file.flush();
                                    }
                                }
                            });
                        },
                        "Open Directory"
                    }
                }
                // Load files buttons row
                div {
                    display: "flex",
                    flex_direction: "row",
                    gap: "10px",
                    button {
                        width: "fit-content",
                        onclick: move |_| {
                            let path = directory_path();
                            spawn(async move {
                                let files = load_audio_files(&path).await;
                                dataset_paths.set(files);
                            });
                        },
                        "Load Audio Files"
                    }
                    button {
                        width: "fit-content",
                        onclick: move |_| {
                            let path = directory_path();
                            spawn(async move {
                                let files = load_image_files(&path).await;
                                dataset_paths.set(files);
                            });
                        },
                        "Load Image Files"
                    }
                }
                // Dataset info display
                if !dataset_paths().is_empty() {
                    div {
                        margin_top: "10px",
                        padding: "8px",
                        background_color: "#2a2a2a",
                        border_radius: "4px",
                        font_size: "11px",
                        div {
                            color: "#4CAF50",
                            "Dataset loaded: {dataset_paths().len()} files"
                        }
                        div {
                            margin_top: "5px",
                            color: "#aaa",
                            font_size: "10px",
                            "Ready for tensor conversion"
                        }
                    }
                }
                // Resize Image Dataset button
                button {
                    width: "fit-content",
                    margin_top: "10px",
                    onclick: move |_| {
                        let path = directory_path();
                        spawn(async move {
                            resize_image_dataset(&path).await;
                        });
                    },
                    "Resize Image Dataset"
                }
                button {
                    width: "fit-content",
                    onclick: move |_| {
                        // Write to log file that serve-all.sh monitors (most reliable)
                        let msg = "[Desktop] ========================================\n[Desktop] Building LSTM model\n[Desktop] ========================================";
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
                    width: "fit-content",
                    onclick: move |_| {
                        // Write to log file that serve-all.sh monitors (most reliable)
                        let msg = "[Desktop] ========================================\n[Desktop] Computing tensor gradients\n[Desktop] ========================================";
                        eprintln!("{}", msg);
                        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
                            use std::io::Write;
                            let _ = writeln!(file, "{}", msg);
                            let _ = file.flush();
                        }
                        
                        spawn(async move {
                            crate::burn_tensor_example();
                            let result_msg = "[Desktop] Tensor gradients computed successfully";
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
        }
    }
}

/// Load and print audio files from directory
/// Returns vector of file paths as strings
#[cfg(feature = "desktop")]
async fn load_audio_files(dir_path: &str) -> Vec<String> {
    use std::path::Path;
    use walkdir::WalkDir;
    
    // Expand ~ in path if present
    let expanded_path = dir_path.replace("~", &std::env::var("HOME").unwrap_or_else(|_| String::new()));
    let path = Path::new(&expanded_path);
    
    if !path.exists() || !path.is_dir() {
        let msg = format!("[Desktop] Error: Directory does not exist: {}", expanded_path);
        eprintln!("{}", msg);
        return Vec::new();
    }
    
    eprintln!("[Desktop] Scanning for audio files in: {}", expanded_path);
    
    // Audio file extensions
    let audio_extensions = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"];
    
    let mut audio_files = Vec::new();
    
    // Walk directory and collect audio files
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if audio_extensions.contains(&ext_lower.as_str()) {
                audio_files.push(entry.path().to_path_buf());
            }
        }
    }
    
    eprintln!("[Desktop] Found {} audio files:", audio_files.len());
    for file_path in &audio_files {
        eprintln!("[Desktop]   - {}", file_path.display());
    }
    
    if audio_files.is_empty() {
        eprintln!("[Desktop] No audio files found in directory.");
        return Vec::new();
    }
    
    // Convert PathBuf to String and return
    audio_files.into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

/// Load and print image files from directory
/// Returns vector of file paths as strings
#[cfg(feature = "desktop")]
async fn load_image_files(dir_path: &str) -> Vec<String> {
    use std::path::Path;
    use walkdir::WalkDir;
    
    // Expand ~ in path if present
    let expanded_path = dir_path.replace("~", &std::env::var("HOME").unwrap_or_else(|_| String::new()));
    let path = Path::new(&expanded_path);
    
    if !path.exists() || !path.is_dir() {
        let msg = format!("[Desktop] Error: Directory does not exist: {}", expanded_path);
        eprintln!("{}", msg);
        return Vec::new();
    }
    
    eprintln!("[Desktop] Scanning for image files in: {}", expanded_path);
    
    // Image file extensions
    let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "svg", "ico"];
    
    let mut image_files = Vec::new();
    
    // Walk directory and collect image files
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if image_extensions.contains(&ext_lower.as_str()) {
                image_files.push(entry.path().to_path_buf());
            }
        }
    }
    
    eprintln!("[Desktop] Found {} image files:", image_files.len());
    for file_path in &image_files {
        eprintln!("[Desktop]   - {}", file_path.display());
    }
    
    if image_files.is_empty() {
        eprintln!("[Desktop] No image files found in directory.");
        return Vec::new();
    }
    
    // Convert PathBuf to String and return
    image_files.into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

/// Resize image dataset - creates a new folder with resized images (512x512)
/// Maintains the same folder structure as the original dataset
/// Uses FFmpeg for fast batch processing (if available) or falls back to image crate
#[cfg(feature = "desktop")]
async fn resize_image_dataset(dir_path: &str) {
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;
    
    // Expand ~ in path if present
    let expanded_path = dir_path.replace("~", &std::env::var("HOME").unwrap_or_else(|_| String::new()));
    let source_path = Path::new(&expanded_path);
    
    if !source_path.exists() || !source_path.is_dir() {
        let msg = format!("[Desktop] Error: Directory does not exist: {}", expanded_path);
        eprintln!("{}", msg);
        return;
    }
    
    // Create output folder name: add "_resized" suffix
    let output_folder_name = format!("{}_resized", source_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("dataset"));
    let output_path = source_path.parent()
        .map(|p| p.join(&output_folder_name))
        .unwrap_or_else(|| PathBuf::from(&output_folder_name));
    
    eprintln!("[Desktop] Starting image resize operation...");
    eprintln!("[Desktop] Source: {}", source_path.display());
    eprintln!("[Desktop] Output: {}", output_path.display());
    
    // Create output directory
    if let Err(e) = std::fs::create_dir_all(&output_path) {
        let msg = format!("[Desktop] Error creating output directory: {}", e);
        eprintln!("{}", msg);
        return;
    }
    
    // Image file extensions
    let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif"];
    
    let mut image_files = Vec::new();
    
    // Collect all image files first
    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
        let source_file = entry.path();
        if let Some(ext) = source_file.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if image_extensions.contains(&ext_lower.as_str()) {
                image_files.push(source_file.to_path_buf());
            }
        }
    }
    
    eprintln!("[Desktop] Found {} images to process", image_files.len());
    
    let mut processed = 0;
    let mut errors = 0;
    
    // Check if FFmpeg binary is available at runtime
    let use_ffmpeg = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok();
    
    if use_ffmpeg {
        eprintln!("[Desktop] Using FFmpeg for fast batch processing...");
        
        for source_file in image_files {
            // Get relative path from source directory
            let relative_path = source_file.strip_prefix(source_path)
                .unwrap_or(&source_file);
            
            // Create output file path maintaining structure
            let output_file = output_path.join(relative_path);
            
            // Create parent directories if needed
            if let Some(parent) = output_file.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("[Desktop] Error creating directory {}: {}", parent.display(), e);
                    errors += 1;
                    continue;
                }
            }
            
            // Use FFmpeg to resize image
            if let Err(e) = resize_image_ffmpeg(&source_file, &output_file, 512, 512).await {
                eprintln!("[Desktop] Error processing {}: {}", source_file.display(), e);
                errors += 1;
            } else {
                processed += 1;
                if processed % 100 == 0 {
                    eprintln!("[Desktop] Processed {} images...", processed);
                }
            }
        }
    } else {
        eprintln!("[Desktop] FFmpeg not found, using image crate (slower but no system dependencies)...");
        use image::ImageReader;
        
        for source_file in image_files {
            // Get relative path from source directory
            let relative_path = source_file.strip_prefix(source_path)
                .unwrap_or(&source_file);
            
            // Create output file path maintaining structure
            let output_file = output_path.join(relative_path);
            
            // Create parent directories if needed
            if let Some(parent) = output_file.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("[Desktop] Error creating directory {}: {}", parent.display(), e);
                    errors += 1;
                    continue;
                }
            }
            
            // Load, resize, and save image using image crate
            match ImageReader::open(&source_file) {
                Ok(reader) => {
                    match reader.decode() {
                        Ok(img) => {
                            // Resize to 512x512 using Lanczos3 filter (high quality)
                            let resized = img.resize_exact(512, 512, image::imageops::FilterType::Lanczos3);
                            
                            // Save resized image
                            if let Err(e) = resized.save(&output_file) {
                                eprintln!("[Desktop] Error saving {}: {}", output_file.display(), e);
                                errors += 1;
                            } else {
                                processed += 1;
                                if processed % 100 == 0 {
                                    eprintln!("[Desktop] Processed {} images...", processed);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[Desktop] Error decoding {}: {}", source_file.display(), e);
                            errors += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Desktop] Error opening {}: {}", source_file.display(), e);
                    errors += 1;
                }
            }
        }
    }
    
    let msg = format!("[Desktop] Resize complete! Processed: {} images, Errors: {}", processed, errors);
    eprintln!("{}", msg);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        use std::io::Write;
        let _ = writeln!(file, "{}", msg);
        let _ = file.flush();
    }
}

/// Resize image using FFmpeg command-line (faster for batch processing)
#[cfg(feature = "desktop")]
async fn resize_image_ffmpeg(input_path: &std::path::Path, output_path: &std::path::Path, width: u32, height: u32) -> Result<(), String> {
    use std::process::Command;
    
    // Use FFmpeg command-line for image resizing (much simpler and faster)
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(format!("scale={}:{}:flags=lanczos", width, height))
        .arg("-y") // Overwrite output file
        .arg(output_path)
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}. Make sure FFmpeg is installed.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg error: {}", stderr));
    }
    
    Ok(())
}

/// Process directory with FFmpeg - scan for audio/video/image files
#[cfg(feature = "desktop")]
async fn process_directory_with_ffmpeg(dir_path: &str) {
    use std::path::Path;
    use walkdir::WalkDir;
    
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        let msg = format!("[Desktop] Error: Directory does not exist: {}", dir_path);
        eprintln!("{}", msg);
        return;
    }
    
    let msg = format!("[Desktop] Scanning directory: {}", dir_path);
    eprintln!("{}", msg);
    
    // Supported media file extensions
    let media_extensions = ["mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "jpg", "jpeg", "png", "gif", "bmp"];
    
    let mut media_files = Vec::new();
    
    // Walk directory and collect media files
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if media_extensions.contains(&ext_lower.as_str()) {
                media_files.push(entry.path().to_path_buf());
            }
        }
    }
    
    let msg = format!("[Desktop] Found {} media files", media_files.len());
    eprintln!("{}", msg);
    
    // Process files with FFmpeg (if available) or just list them
    #[cfg(feature = "ffmpeg")]
    {
        // FFmpeg processing would go here
        for file_path in media_files.iter().take(10) { // Limit to first 10 for now
            let msg = format!("[Desktop] Processing with FFmpeg: {}", file_path.display());
            eprintln!("{}", msg);
            // TODO: Use FFmpeg to decode files into arrays/tensors for training
        }
    }
    
    #[cfg(not(feature = "ffmpeg"))]
    {
        // Just list files if FFmpeg is not available
        for file_path in media_files.iter().take(10) {
            let msg = format!("[Desktop] Found media file: {}", file_path.display());
            eprintln!("{}", msg);
        }
        eprintln!("[Desktop] Note: FFmpeg feature not enabled. Install FFmpeg system libraries and enable 'ffmpeg' feature to process files.");
    }
    
    let msg = format!("[Desktop] Directory processing complete. Found {} media files.", media_files.len());
    eprintln!("{}", msg);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        use std::io::Write;
        let _ = writeln!(file, "{}", msg);
        let _ = file.flush();
    }
}

/// Echo component that demonstrates fullstack server functions (Desktop)
#[cfg(feature = "desktop")]
#[component]
fn DesktopEcho() -> Element {
    let mut response = use_signal(|| String::new());
    let mut error = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            div { font_size: "12px", "ServerFn Echo" }
            br {}
            input {
                placeholder: "Type here to echo.",
                oninput:  move |event| async move {
                    error.set(String::new());
                    match echo_server(event.value()).await {
                        Ok(data) => {
                            response.set(data);
                        }
                        Err(e) => {
                            error.set(format!("Error: {}", e));
                            response.set(String::new());
                        }
                    }
                },
            }

            if !error().is_empty() {
                div {
                    color: "#ff6b6b",
                    font_size: "10px",
                    margin_top: "5px",
                    "{error}"
                }
            }

            if !response().is_empty() {
                p {
                    font_size: "12px",
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

