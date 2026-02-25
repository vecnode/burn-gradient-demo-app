use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::sync::OnceLock;

// ============================================================================
// Agent Actor Implementation
// ============================================================================

/// Generic agent actor that can be instantiated with different IDs
pub struct Agent {
    /// The unique identifier for this agent (1-5)
    pub id: u8,
}

/// Message types that agents can handle
#[derive(Debug, Clone)]
pub enum AgentMessage {
    /// Process data asynchronously
    ProcessData {
        data: String,
    },
    /// Get the current status of the agent
    GetStatus,
    /// Custom action with parameters
    CustomAction {
        action: String,
        params: Vec<String>,
    },
}

/// Agent state - maintains internal state for each agent
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Agent identifier
    pub id: u8,
    /// Counter for processed messages
    pub processed_count: u64,
    /// Last processed data
    pub last_data: Option<String>,
}

impl Actor for Agent {
    type Msg = AgentMessage;
    type State = AgentState;
    type Arguments = u8; // Agent ID

    /// Initialize the agent with its ID
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        agent_id: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        eprintln!("[Agent{}] Starting agent with ID {}", agent_id, agent_id);
        Ok(AgentState {
            id: agent_id,
            processed_count: 0,
            last_data: None,
        })
    }

    /// Handle messages asynchronously
    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            AgentMessage::ProcessData { data } => {
                state.processed_count += 1;
                state.last_data = Some(data.clone());
                
                // Print with agent identifier (1, 2, 3, 4, or 5)
                eprintln!("[Agent{}] Processing data: '{}' | Total processed: {}", 
                    state.id, data, state.processed_count);
                
                // Simulate async I/O operation
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            AgentMessage::GetStatus => {
                eprintln!("[Agent{}] Status - Processed: {} messages, Last data: {:?}", 
                    state.id, state.processed_count, state.last_data);
            }
            AgentMessage::CustomAction { action, params } => {
                eprintln!("[Agent{}] Custom action: '{}' with params: {:?}", 
                    state.id, action, params);
                state.processed_count += 1;
            }
        }
        Ok(())
    }
}

// ============================================================================
// Actor Registry
// ============================================================================

/// Registry to store references to all 5 agents
/// Using individual OnceLock for type safety
pub static AGENT_1: OnceLock<ActorRef<AgentMessage>> = OnceLock::new();
pub static AGENT_2: OnceLock<ActorRef<AgentMessage>> = OnceLock::new();
pub static AGENT_3: OnceLock<ActorRef<AgentMessage>> = OnceLock::new();
pub static AGENT_4: OnceLock<ActorRef<AgentMessage>> = OnceLock::new();
pub static AGENT_5: OnceLock<ActorRef<AgentMessage>> = OnceLock::new();

/// Initialize all 5 agents using the provided Tokio runtime
/// This should be called once at application startup
/// Uses a static flag to ensure it only runs once
static INIT_FLAG: std::sync::OnceLock<tokio::sync::Mutex<bool>> = std::sync::OnceLock::new();

pub async fn initialize_agents() -> Result<(), Box<dyn std::error::Error>> {
    let init_mutex = INIT_FLAG.get_or_init(|| tokio::sync::Mutex::new(false));
    let mut initialized = init_mutex.lock().await;
    
    if *initialized {
        return Ok(()); // Already initialized
    }
    eprintln!("[AgentRegistry] Initializing all 5 agents...");
    
    // Spawn all agents concurrently
    let (actor1_ref, handle1) = Actor::spawn(None, Agent { id: 1 }, 1)
        .await
        .map_err(|e| format!("Failed to spawn Agent1: {:?}", e))?;
    
    let (actor2_ref, handle2) = Actor::spawn(None, Agent { id: 2 }, 2)
        .await
        .map_err(|e| format!("Failed to spawn Agent2: {:?}", e))?;
    
    let (actor3_ref, handle3) = Actor::spawn(None, Agent { id: 3 }, 3)
        .await
        .map_err(|e| format!("Failed to spawn Agent3: {:?}", e))?;
    
    let (actor4_ref, handle4) = Actor::spawn(None, Agent { id: 4 }, 4)
        .await
        .map_err(|e| format!("Failed to spawn Agent4: {:?}", e))?;
    
    let (actor5_ref, handle5) = Actor::spawn(None, Agent { id: 5 }, 5)
        .await
        .map_err(|e| format!("Failed to spawn Agent5: {:?}", e))?;
    
    // Store references in the registry
    AGENT_1.set(actor1_ref.clone())
        .map_err(|_| "Failed to store Agent1 reference")?;
    AGENT_2.set(actor2_ref.clone())
        .map_err(|_| "Failed to store Agent2 reference")?;
    AGENT_3.set(actor3_ref.clone())
        .map_err(|_| "Failed to store Agent3 reference")?;
    AGENT_4.set(actor4_ref.clone())
        .map_err(|_| "Failed to store Agent4 reference")?;
    AGENT_5.set(actor5_ref.clone())
        .map_err(|_| "Failed to store Agent5 reference")?;
    
    eprintln!("[AgentRegistry] All 5 agents initialized successfully!");
    
    // Mark as initialized
    *initialized = true;
    
    // Spawn tasks to keep actors alive (they run in the background)
    tokio::spawn(async move {
        let _ = handle1.await;
    });
    tokio::spawn(async move {
        let _ = handle2.await;
    });
    tokio::spawn(async move {
        let _ = handle3.await;
    });
    tokio::spawn(async move {
        let _ = handle4.await;
    });
    tokio::spawn(async move {
        let _ = handle5.await;
    });
    
    Ok(())
}

/// Ensure agents are initialized (lazy initialization)
/// Call this from server functions to ensure agents are ready
pub async fn ensure_agents_initialized() -> Result<(), Box<dyn std::error::Error>> {
    if AGENT_1.get().is_none() {
        initialize_agents().await?;
    }
    Ok(())
}

/// Get actor reference by ID (1-5)
pub fn get_agent(agent_id: u8) -> Option<ActorRef<AgentMessage>> {
    match agent_id {
        1 => AGENT_1.get().cloned(),
        2 => AGENT_2.get().cloned(),
        3 => AGENT_3.get().cloned(),
        4 => AGENT_4.get().cloned(),
        5 => AGENT_5.get().cloned(),
        _ => None,
    }
}
