use crate::domain::events::{BoardEvent, EventBus};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info, instrument, warn};
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
                debug!("Creating new broadcast channel");
                broadcast::channel(CHANNEL_CAPACITY).0
            })
            .clone()
    }

    async fn cleanup_if_empty(&self, board_id: Uuid) {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(&board_id)
            && sender.receiver_count() == 0
        {
            debug!("Cleaning up empty broadcast channel");
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
    #[instrument(
        name = "event_bus.publish",
        skip(self, board_id, event),
        fields(
            board.id = %board_id,
            event.type = %event
        )
    )]
    async fn publish(&self, board_id: Uuid, event: BoardEvent) {
        let sender = self.get_or_create_channel(board_id).await;

        if sender.receiver_count() == 0 {
            debug!("No subscribers, event will not be published");
            return;
        }

        match sender.send(event.clone()) {
            Ok(count) => info!(event.subscribers = %count, "Event published successfully"),
            Err(err) => warn!(error = %err, "Failed to publish event"),
        }
    }

    #[instrument(
        name = "event_bus.subscribe",
        skip(self, board_id),
        fields(board.id = %board_id)
    )]
    async fn subscribe(&self, board_id: Uuid) -> broadcast::Receiver<BoardEvent> {
        debug!("Creating new subscription for board events");
        let sender = self.get_or_create_channel(board_id).await;

        sender.subscribe()
    }

    #[instrument(
        name = "event_bus.cleanup_board",
        skip(self, board_id),
        fields(board.id = %board_id)
    )]
    async fn cleanup_board(&self, board_id: Uuid) {
        debug!("Cleaning up board event channel");
        self.cleanup_if_empty(board_id).await;
    }
}
