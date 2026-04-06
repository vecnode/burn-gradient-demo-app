use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::sync::OnceLock;

pub struct GradientActor;

#[derive(Debug, Clone)]
pub enum ActorMessage {
    ComputeTensorGradients,
}

#[derive(Debug, Clone)]
pub struct GradientActorState {
    pub processed_count: u64,
}

impl Actor for GradientActor {
    type Msg = ActorMessage;
    type State = GradientActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        eprintln!("[GradientActor] Started");
        Ok(GradientActorState { processed_count: 0 })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ActorMessage::ComputeTensorGradients => {
                state.processed_count += 1;
                eprintln!(
                    "[GradientActor] Running tensor gradient computation #{}",
                    state.processed_count
                );
                crate::burn_tensor_example();
            }
        }
        Ok(())
    }
}

pub static GRADIENT_ACTOR: OnceLock<ActorRef<ActorMessage>> = OnceLock::new();
static INIT_FLAG: std::sync::OnceLock<tokio::sync::Mutex<bool>> = std::sync::OnceLock::new();

pub async fn initialize_actors() -> Result<(), Box<dyn std::error::Error>> {
    let init_mutex = INIT_FLAG.get_or_init(|| tokio::sync::Mutex::new(false));
    let mut initialized = init_mutex.lock().await;

    if *initialized {
        return Ok(());
    }

    eprintln!("[ActorRegistry] Initializing gradient actor...");
    let (actor_ref, handle) = Actor::spawn(None, GradientActor, ())
        .await
        .map_err(|e| format!("Failed to spawn gradient actor: {:?}", e))?;

    GRADIENT_ACTOR
        .set(actor_ref)
        .map_err(|_| "Failed to store gradient actor reference")?;

    *initialized = true;
    eprintln!("[ActorRegistry] Gradient actor initialized successfully");

    tokio::spawn(async move {
        let _ = handle.await;
    });

    Ok(())
}

pub async fn ensure_actors_initialized() -> Result<(), Box<dyn std::error::Error>> {
    if GRADIENT_ACTOR.get().is_none() {
        initialize_actors().await?;
    }
    Ok(())
}

pub async fn trigger_gradient_computation() -> Result<(), Box<dyn std::error::Error>> {
    ensure_actors_initialized().await?;
    if let Some(actor_ref) = GRADIENT_ACTOR.get() {
        let _ = actor_ref.send_message(ActorMessage::ComputeTensorGradients);
        Ok(())
    } else {
        Err("Gradient actor not available".into())
    }
}