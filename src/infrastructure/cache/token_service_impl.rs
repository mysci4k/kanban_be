use crate::domain::services::TokenService;
use crate::shared::utils::constants::{ACTIVATION_TOKEN_TTL, PASSWORD_RESET_TOKEN_TTL};
use async_trait::async_trait;
use redis::Client as RedisClient;
use redis::{AsyncTypedCommands, RedisError};

pub struct RedisTokenService {
    redis_client: RedisClient,
}

impl RedisTokenService {
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }

    fn user_activation_key(&self, user_id: &str) -> String {
        format!("user_activation:{}", user_id)
    }

    fn user_password_reset_key(&self, user_id: &str) -> String {
        format!("user_password_reset:{}", user_id)
    }
}

#[async_trait]
impl TokenService for RedisTokenService {
    async fn store_activation_token(
        &self,
        user_id: &str,
        activation_token: &str,
    ) -> Result<(), String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_activation_key(user_id);
        conn.set_ex(&key, activation_token, *ACTIVATION_TOKEN_TTL)
            .await
            .map_err(|err: RedisError| format!("Failed to store token: {}", err))?;

        Ok(())
    }

    async fn validate_activation_token(
        &self,
        user_id: &str,
        activation_token: &str,
    ) -> Result<bool, String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_activation_key(user_id);
        let stored_token: Option<String> = conn
            .get(&key)
            .await
            .map_err(|err: RedisError| format!("Failed to get token: {}", err))?;

        Ok(stored_token.as_deref() == Some(activation_token))
    }

    async fn has_active_token(&self, user_id: &str) -> Result<bool, String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_activation_key(user_id);

        conn.exists(&key)
            .await
            .map_err(|err: RedisError| format!("Failed to check token existence: {}", err))
    }

    async fn delete_activation_token(&self, user_id: &str) -> Result<(), String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_activation_key(user_id);
        conn.del(&key)
            .await
            .map_err(|err: RedisError| format!("Failed to delete token: {}", err))?;

        Ok(())
    }

    async fn store_password_reset_token(
        &self,
        user_id: &str,
        reset_token: &str,
    ) -> Result<(), String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_password_reset_key(user_id);
        conn.set_ex(&key, reset_token, *PASSWORD_RESET_TOKEN_TTL)
            .await
            .map_err(|err: RedisError| format!("Failed to store token: {}", err))?;

        Ok(())
    }

    async fn validate_password_reset_token(
        &self,
        user_id: &str,
        reset_token: &str,
    ) -> Result<bool, String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_password_reset_key(user_id);
        let stored_token: Option<String> = conn
            .get(&key)
            .await
            .map_err(|err: RedisError| format!("Failed to get token: {}", err))?;

        Ok(stored_token.as_deref() == Some(reset_token))
    }

    async fn has_active_password_reset_token(&self, user_id: &str) -> Result<bool, String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_password_reset_key(user_id);

        conn.exists(&key)
            .await
            .map_err(|err: RedisError| format!("Failed to check token existence: {}", err))
    }

    async fn delete_password_reset_token(&self, user_id: &str) -> Result<(), String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("Redis connection error: {}", err))?;

        let key = self.user_password_reset_key(user_id);
        conn.del(&key)
            .await
            .map_err(|err: RedisError| format!("Failed to delete token: {}", err))?;

        Ok(())
    }
}
