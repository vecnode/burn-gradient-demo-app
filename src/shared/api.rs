// Shared API server functions

use dioxus::prelude::*;
use crate::agents::{get_agent, ensure_agents_initialized};

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo_server(input: String) -> Result<String, ServerFnError> {
    // Ensure agents are initialized
    let _ = ensure_agents_initialized().await;
    
    // For backward compatibility, send to Agent1
    if let Some(actor_ref) = get_agent(1) {
        use crate::agents::AgentMessage;
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: input.clone(),
        });
    }
    
    Ok(input)
}

// ============================================================================
// HTTP/REST API Endpoints for Multi-Agent System
// ============================================================================

/// Process data through Agent 1
#[post("/api/agents/1/process")]
pub async fn process_agent1(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(1) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent1: {}", data))
    } else {
        Err(ServerFnError::new("Agent1 is not available"))
    }
}

/// Process data through Agent 2
#[post("/api/agents/2/process")]
pub async fn process_agent2(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(2) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent2: {}", data))
    } else {
        Err(ServerFnError::new("Agent2 is not available"))
    }
}

/// Process data through Agent 3
#[post("/api/agents/3/process")]
pub async fn process_agent3(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(3) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent3: {}", data))
    } else {
        Err(ServerFnError::new("Agent3 is not available"))
    }
}

/// Process data through Agent 4
#[post("/api/agents/4/process")]
pub async fn process_agent4(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(4) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent4: {}", data))
    } else {
        Err(ServerFnError::new("Agent4 is not available"))
    }
}

/// Process data through Agent 5
#[post("/api/agents/5/process")]
pub async fn process_agent5(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(5) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent5: {}", data))
    } else {
        Err(ServerFnError::new("Agent5 is not available"))
    }
}

/// Get status of a specific agent
#[get("/api/agents/:id/status")]
pub async fn get_agent_status(id: u8) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(id) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::GetStatus);
        Ok(format!("Status request sent to Agent{}", id))
    } else {
        Err(ServerFnError::new(format!("Agent{} is not available", id)))
    }
}

/// Process data through any agent (dynamic routing)
#[post("/api/agents/:id/process")]
pub async fn process_agent_dynamic(id: u8, data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(id) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent{}: {}", id, data))
    } else {
        Err(ServerFnError::new(format!("Agent{} is not available", id)))
    }
}

