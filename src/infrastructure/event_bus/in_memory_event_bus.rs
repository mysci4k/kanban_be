use crate::domain::events::{BoardEvent, EventBus};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn};
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 100;

pub struct InMemoryEventBus {
    channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<BoardEvent>>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_channel(&self, board_id: Uuid) -> broadcast::Sender<BoardEvent> {
        let mut channels = self.channels.write().await;

        channels
            .entry(board_id)
            .or_insert_with(|| {
                info!("Creating new broadcast channel for board '{}'", board_id);
                broadcast::channel(CHANNEL_CAPACITY).0
            })
            .clone()
    }

    async fn cleanup_if_empty(&self, board_id: Uuid) {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(&board_id)
            && sender.receiver_count() == 0
        {
            info!(
                "Cleaning up empty broadcast channel for board '{}'",
                board_id
            );

            channels.remove(&board_id);
        }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, board_id: Uuid, event: BoardEvent) {
        let sender = self.get_or_create_channel(board_id).await;

        if sender.receiver_count() == 0 {
            info!(
                "No subscribers for board '{}', event will not be published",
                board_id
            );

            return;
        }

        match sender.send(event.clone()) {
            Ok(count) => info!(
                "Published event to board '{}', {} subscribers notified: {:?}",
                board_id, count, event
            ),
            Err(err) => warn!("Failed to publish event to board '{}': {}", board_id, err),
        }
    }

    async fn subscribe(&self, board_id: Uuid) -> broadcast::Receiver<BoardEvent> {
        let sender = self.get_or_create_channel(board_id).await;

        sender.subscribe()
    }

    async fn cleanup_board(&self, board_id: Uuid) {
        self.cleanup_if_empty(board_id).await;
    }
}
