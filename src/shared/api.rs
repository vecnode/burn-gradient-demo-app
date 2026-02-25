// Shared API server functions

use dioxus::prelude::*;

/// Echo the user input on the server.
/// Calls agent 1 directly (same process, no forwarding needed)
#[post("/api/echo")]
pub async fn echo_server(input: String) -> Result<String, ServerFnError> {
    // Call agent 1 directly (same process)
    use crate::agents::{get_agent, AgentMessage};
    if let Some(actor_ref) = get_agent(1) {
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: input.clone(),
        });
    }
    Ok(input)
}
