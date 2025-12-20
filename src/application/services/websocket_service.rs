use crate::{
    domain::{
        events::{BoardEvent, SharedEventBus},
        repositories::BoardMemberRepository,
    },
    shared::error::ApplicationError,
};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};
use uuid::Uuid;

pub struct WebSocketService {
    event_bus: SharedEventBus,
    board_member_repository: Arc<dyn BoardMemberRepository>,
}

impl WebSocketService {
    pub fn new(
        event_bus: SharedEventBus,
        board_member_repository: Arc<dyn BoardMemberRepository>,
    ) -> Self {
        Self {
            event_bus,
            board_member_repository,
        }
    }

    pub async fn verify_board_access(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ApplicationError> {
        let is_member = self
            .board_member_repository
            .find_by_board_and_user_id(board_id, user_id)
            .await?
            .is_some();

        if !is_member {
            return Err(ApplicationError::Forbidden {
                message: "You don't have access to this board".to_string(),
            });
        }

        Ok(())
    }

    pub async fn subscribe_to_board(&self, board_id: Uuid) -> broadcast::Receiver<BoardEvent> {
        self.event_bus.subscribe(board_id).await
    }

    pub async fn handle_connection(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        mut session: Session,
        mut msg_stream: MessageStream,
    ) {
        info!("User '{}' connected to board '{}'", user_id, board_id);

        let mut rx = self.subscribe_to_board(board_id).await;

        loop {
            tokio::select! {
                event = rx.recv() => {
                    match event {
                        Ok(event) => {
                            if let Ok(json) = serde_json::to_string(&event)
                                && session.text(json).await.is_err() {
                                break;
                            }

                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Client lagged '{}' messages", n);
                        }
                        Err(_) => break,
                    }
                }

                msg = msg_stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
            }
        }

        drop(rx);

        info!("User '{}' disconnected from board '{}'", user_id, board_id);

        self.event_bus.cleanup_board(board_id).await;
    }
}
