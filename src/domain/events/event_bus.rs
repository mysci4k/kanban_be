use crate::domain::events::BoardEvent;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, board_id: Uuid, event: BoardEvent);
    async fn subscribe(&self, board_id: Uuid) -> broadcast::Receiver<BoardEvent>;
    async fn cleanup_board(&self, board_id: Uuid);
}

pub type SharedEventBus = Arc<dyn EventBus>;
