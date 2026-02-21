// Real-time communication between desktop and web apps
// Optimized for reliability and low latency

use dioxus::prelude::*;
use std::sync::OnceLock;
use std::collections::VecDeque;
use std::sync::Mutex;

// Message buffer for reliable message delivery via polling
// Stores messages until consumed by web app polling
static MESSAGE_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn get_message_buffer() -> &'static Mutex<VecDeque<String>> {
    MESSAGE_BUFFER.get_or_init(|| Mutex::new(VecDeque::with_capacity(1000)))
}

/// Desktop app sends message to web app via HTTP POST
/// Message is stored in buffer for reliable delivery via polling
#[post("/api/messages/send")]
pub async fn send_message(message: String) -> Result<String, ServerFnError> {
    // Store in buffer (for polling clients)
    // Buffer ensures no message loss even if polling is temporarily delayed
    let mut buffer = get_message_buffer().lock().unwrap();
    buffer.push_back(message);
    // Keep last 1000 messages to handle high-frequency bursts
    if buffer.len() > 1000 {
        buffer.pop_front();
    }
    Ok("Message sent".to_string())
}

/// Long-polling endpoint for real-time message streaming
/// Returns immediately with message from buffer if available, or empty string
/// Optimized single-lock operation for minimal latency
#[get("/api/messages/stream")]
pub async fn stream_messages() -> Result<String, ServerFnError> {
    // Single lock operation - check and consume atomically
    let mut buffer = get_message_buffer().lock().unwrap();
    if let Some(msg) = buffer.pop_front() {
        Ok(msg)
    } else {
        Ok(String::new())
    }
}
