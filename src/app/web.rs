// Web application component

#[cfg(any(feature = "web", feature = "server"))]
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(any(feature = "web", feature = "server"))]
const FAVICON: Asset = asset!("/assets/favicon.ico");

/// Web application root component
/// Receives messages from desktop app via SSE (Server-Sent Events)
#[cfg(any(feature = "web", feature = "server"))]
#[component]
pub fn WebApp() -> Element {
    let messages = use_signal(|| Vec::<String>::new());
    
    // Initialize vis-timeline with background areas
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            
            spawn(async move {
                // Wait for DOM and vis library to be ready
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    let window = web_sys::window().unwrap();
                    let closure = Closure::wrap(Box::new(move || {
                        resolve.call0(&JsValue::UNDEFINED).unwrap();
                    }) as Box<dyn FnMut()>);
                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        500
                    ).unwrap();
                    closure.forget();
                });
                let _ = JsFuture::from(promise).await;
                
                // Wait for vis library to be available
                let window = web_sys::window().unwrap();
                let mut attempts = 0;
                loop {
                    let vis_available = js_sys::Reflect::has(&window, &JsValue::from_str("vis")).unwrap_or(false);
                    if vis_available || attempts > 20 {
                        break;
                    }
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let window = web_sys::window().unwrap();
                        let closure = Closure::wrap(Box::new(move || {
                            resolve.call0(&JsValue::UNDEFINED).unwrap();
                        }) as Box<dyn FnMut()>);
                        window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            100
                        ).unwrap();
                        closure.forget();
                    });
                    let _ = JsFuture::from(promise).await;
                    attempts += 1;
                }
                
                // Initialize timeline with background areas
                let document = window.document().unwrap();
                if let Some(container) = document.get_element_by_id("visualization") {
                    if let Ok(vis_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("vis")) {
                        // Create items array
                        let items = js_sys::Array::new();
                        
                        // Regular items
                        let item1 = js_sys::Object::new();
                        js_sys::Reflect::set(&item1, &JsValue::from_str("id"), &JsValue::from_str("1")).unwrap();
                        js_sys::Reflect::set(&item1, &JsValue::from_str("content"), &JsValue::from_str("item 1")).unwrap();
                        js_sys::Reflect::set(&item1, &JsValue::from_str("start"), &JsValue::from_str("2014-04-20")).unwrap();
                        items.push(&item1);
                        
                        let item2 = js_sys::Object::new();
                        js_sys::Reflect::set(&item2, &JsValue::from_str("id"), &JsValue::from_str("2")).unwrap();
                        js_sys::Reflect::set(&item2, &JsValue::from_str("content"), &JsValue::from_str("item 2")).unwrap();
                        js_sys::Reflect::set(&item2, &JsValue::from_str("start"), &JsValue::from_str("2014-04-14")).unwrap();
                        items.push(&item2);
                        
                        // Background areas (Period A and Period B)
                        let bg1 = js_sys::Object::new();
                        js_sys::Reflect::set(&bg1, &JsValue::from_str("id"), &JsValue::from_str("bg1")).unwrap();
                        js_sys::Reflect::set(&bg1, &JsValue::from_str("type"), &JsValue::from_str("background")).unwrap();
                        js_sys::Reflect::set(&bg1, &JsValue::from_str("content"), &JsValue::from_str("Period A")).unwrap();
                        js_sys::Reflect::set(&bg1, &JsValue::from_str("start"), &JsValue::from_str("2014-04-10")).unwrap();
                        js_sys::Reflect::set(&bg1, &JsValue::from_str("end"), &JsValue::from_str("2014-04-16")).unwrap();
                        items.push(&bg1);
                        
                        let bg2 = js_sys::Object::new();
                        js_sys::Reflect::set(&bg2, &JsValue::from_str("id"), &JsValue::from_str("bg2")).unwrap();
                        js_sys::Reflect::set(&bg2, &JsValue::from_str("type"), &JsValue::from_str("background")).unwrap();
                        js_sys::Reflect::set(&bg2, &JsValue::from_str("content"), &JsValue::from_str("Period B")).unwrap();
                        js_sys::Reflect::set(&bg2, &JsValue::from_str("start"), &JsValue::from_str("2014-04-18")).unwrap();
                        js_sys::Reflect::set(&bg2, &JsValue::from_str("end"), &JsValue::from_str("2014-04-25")).unwrap();
                        items.push(&bg2);
                        
                        // Create DataSet - following the pattern from index.html.backup
                        // new vis.DataSet([...]) becomes: vis.DataSet.construct([...])
                        if let Ok(DataSet_val) = js_sys::Reflect::get(&vis_obj, &JsValue::from_str("DataSet")) {
                            if let Some(DataSet) = DataSet_val.dyn_ref::<js_sys::Function>() {
                                let data_set_args = js_sys::Array::new();
                                data_set_args.push(&items);
                                if let Ok(data_set) = js_sys::Reflect::construct(DataSet, &data_set_args) {
                                    // Create Timeline - following: new vis.Timeline(container, items, options)
                                    if let Ok(Timeline_val) = js_sys::Reflect::get(&vis_obj, &JsValue::from_str("Timeline")) {
                                        if let Some(Timeline) = Timeline_val.dyn_ref::<js_sys::Function>() {
                                            let args = js_sys::Array::new();
                                            args.push(&container.into());
                                            args.push(&data_set.into());
                                            args.push(&js_sys::Object::new().into());
                                            let _ = js_sys::Reflect::construct(Timeline, &args);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    
    // Long-polling: continuously poll for messages from desktop app
    use_effect(move || {
        spawn(async move {
            let mut msg_list = messages;
            
            // Wait for server to be ready (avoid 500 errors during startup)
            #[cfg(target_arch = "wasm32")]
            {
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    let window = web_sys::window().unwrap();
                    let closure = Closure::wrap(Box::new(move || {
                        resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap();
                    }) as Box<dyn FnMut()>);
                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        3000 // Wait 3 seconds for server to be ready
                    ).unwrap();
                    closure.forget();
                });
                let _ = JsFuture::from(promise).await;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
            
            loop {
                // Poll for messages (returns immediately with message or empty string)
                match crate::shared::stream_messages().await {
                    Ok(msg) if !msg.is_empty() => {
                        // Add message to list (no limit - keep all messages)
                        msg_list.with_mut(|r| {
                            r.push(msg.trim().to_string());
                        });
                    }
                    Ok(_) => {
                        // Empty message - continue polling
                    }
                    Err(_) => {
                        // Server not ready or error - silently retry (don't flood logs)
                        // This handles 500 errors during startup gracefully
                    }
                }
                // Small delay to prevent excessive CPU usage (WASM-compatible)
                #[cfg(target_arch = "wasm32")]
                {
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let window = web_sys::window().unwrap();
                        let closure = Closure::wrap(Box::new(move || {
                            resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap();
                        }) as Box<dyn FnMut()>);
                        window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            10
                        ).unwrap();
                        closure.forget();
                    });
                    let _ = JsFuture::from(promise).await;
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    tokio::task::yield_now().await;
                }
            }
        });
    });
    
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Script { src: asset!("/assets/vis-timeline-graph2d.min.js") }
        document::Link { rel: "stylesheet", href: asset!("/assets/vis-timeline-graph2d.min.css") }
        div {
            font_family: "system-ui, sans-serif",
            header {
                h1 { "pattern-clock" }
                div { "Web Interface - Real-time Communication" }
            }
            div {
                margin_top: "20px",
                h3 { "Timeline" }
                div {
                    id: "visualization",
                    width: "100%",
                    height: "300px",
                    border: "1px solid #bfbfbf",
                    border_radius: "5px",
                }
            }
            div {
                margin_top: "20px",
                h3 { 
                    "Messages (from Desktop App): "
                    span {
                        color: if messages().is_empty() { "#999" } else { "#4caf50" },
                        "({messages().len()} received)"
                    }
                }
            }
        }
    }
}
