// Shared API server functions

use dioxus::prelude::*;

/// Echo the user input on the server.
/// Calls agent 1 directly (same process, no forwarding needed)
#[post("/api/echo")]
pub async fn echo_server(input: String) -> Result<String, ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Call agent 1 directly (same process)
        use crate::agents::{get_agent, AgentMessage};
    if let Some(actor_ref) = get_agent(1) {
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: input.clone(),
        });
    }
    Ok(input)
}
    #[cfg(target_arch = "wasm32")]
    {
        // WASM stub - this should never be called on client
        Ok(input)
    }
}
