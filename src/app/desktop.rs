#[cfg(feature = "desktop")]
use dioxus::prelude::*;
#[cfg(feature = "desktop")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "desktop")]
use crate::actors::{ensure_actors_initialized, trigger_gradient_computation};

#[cfg(feature = "desktop")]
static INITIALIZATION_STARTED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "desktop")]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(feature = "desktop")]
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[cfg(feature = "desktop")]
#[component]
pub fn DesktopApp() -> Element {
    use_effect(move || {
        spawn(async move {
            if INITIALIZATION_STARTED.swap(true, Ordering::SeqCst) {
                return;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            if let Err(e) = ensure_actors_initialized().await {
                eprintln!("[Desktop] Failed to initialize actors: {}", e);
            } else {
                eprintln!("[Desktop] Actors initialized successfully");
            }
        });
    });
    
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        
        div {
            padding: "20px",
            MLOperationsTab {}
        }
    }
}

#[cfg(feature = "desktop")]
#[component]
fn MLOperationsTab() -> Element {
    rsx! {
        div {
            div { font_size: "12px", "ML Operations" }
            br {}
            div {
                display: "flex",
                flex_direction: "column",
                gap: "10px",
                width: "fit-content",
                button {
                    width: "fit-content",
                    onclick: move |_| {
                        let msg = "[Desktop] ========================================\n[Desktop] Computing tensor gradients\n[Desktop] ========================================";
                        eprintln!("{}", msg);
                        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/burn-gradient-demo-app-desktop.log") {
                            use std::io::Write;
                            let _ = writeln!(file, "{}", msg);
                            let _ = file.flush();
                        }
                        
                        spawn(async move {
                            match trigger_gradient_computation().await {
                                Ok(()) => {
                                    let result_msg = "[Desktop] Tensor gradient job queued to actor";
                                    eprintln!("{}", result_msg);
                                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/burn-gradient-demo-app-desktop.log") {
                                        use std::io::Write;
                                        let _ = writeln!(file, "{}", result_msg);
                                        let _ = file.flush();
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[Desktop] Failed to queue gradient job: {}", e);
                                }
                            }
                        });
                    },
                    "Compute Tensor Gradients"
                }
            }
        }
    }
}

