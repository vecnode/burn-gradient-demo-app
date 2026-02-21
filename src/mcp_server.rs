use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
};
use crate::agents::{get_agent, ensure_agents_initialized, AgentMessage};

pub struct PatternClockMCP {
    tool_router: ToolRouter<PatternClockMCP>,
}

#[tool_router]
impl PatternClockMCP {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Example tool that returns a greeting
    #[tool(description = "A simple example tool that returns a greeting message")]
    pub async fn example_tool(&self) -> String {
        "Hello from pattern-clock MCP server!".to_string()
    }

    /// Get random numbers
    #[tool(description = "Returns a random number between 0 and 1000")]
    pub async fn get_random_number(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut hasher = DefaultHasher::new();
        timestamp.hash(&mut hasher);
        let hash = hasher.finish();
        
        let random = (hash % 1000) as u64;
        format!("Random number: {}", random)
    }
}

#[tool_handler]
impl ServerHandler for PatternClockMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// Public functions that can be called directly from the app (not just via MCP protocol)
impl PatternClockMCP {
    /// Call the example tool directly (for use by desktop app)
    pub async fn call_example_tool(&self) -> String {
        self.example_tool().await
    }

    /// Get random number directly (for use by desktop app)
    pub async fn call_get_random_number(&self) -> String {
        self.get_random_number().await
    }
}
