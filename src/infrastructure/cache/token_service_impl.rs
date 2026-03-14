use crate::domain::services::TokenService;
use crate::shared::utils::constants::{ACTIVATION_TOKEN_TTL, PASSWORD_RESET_TOKEN_TTL};
use async_trait::async_trait;
use redis::Client as RedisClient;
use redis::{AsyncTypedCommands, RedisError};
use subtle::ConstantTimeEq;
use tracing::{debug, error, instrument};

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
    #[instrument(
        name = "redis.store_activation_token",
        skip(self, user_id, activation_token),
        fields(
            db.operation = "set",
            user.id = %user_id
        ),
        err
    )]
    async fn store_activation_token(
        &self,
        user_id: &str,
        activation_token: &str,
    ) -> Result<(), String> {
        debug!("Storing activation token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_activation_key(user_id);
        conn.set_ex(&key, activation_token, *ACTIVATION_TOKEN_TTL)
            .await
            .map_err(|err: RedisError| {
                error!(error = %err, "Failed to store activation token");
                format!("Failed to store token: {}", err)
            })?;

        Ok(())
    }

    #[instrument(
        name = "redis.validate_activation_token",
        skip(self, user_id, activation_token),
        fields(
            db.operation = "get",
            user.id = %user_id
        ),
        err
    )]
    async fn validate_activation_token(
        &self,
        user_id: &str,
        activation_token: &str,
    ) -> Result<bool, String> {
        debug!("Validating activation token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_activation_key(user_id);
        let stored_token: Option<String> = conn.get(&key).await.map_err(|err: RedisError| {
            error!(error = %err, "Failed to get activation token");
            format!("Failed to get token: {}", err)
        })?;

        let is_valid = stored_token
            .as_ref()
            .map(|stored| stored.as_bytes().ct_eq(activation_token.as_bytes()).into())
            .unwrap_or(false);

        Ok(is_valid)
    }

    #[instrument(
        name = "redis.has_active_token",
        skip(self, user_id),
        fields(
            db.operation = "exists",
            user.id = %user_id
        ),
        err
    )]
    async fn has_active_token(&self, user_id: &str) -> Result<bool, String> {
        debug!("Checking for active activation token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_activation_key(user_id);

        conn.exists(&key).await.map_err(|err: RedisError| {
            error!(error = %err, "Failed to check activation token existence");
            format!("Failed to check token existence: {}", err)
        })
    }

    #[instrument(
        name = "redis.delete_activation_token",
        skip(self, user_id),
        fields(
            db.operation = "delete",
            user.id = %user_id
        ),
        err
    )]
    async fn delete_activation_token(&self, user_id: &str) -> Result<(), String> {
        debug!("Deleting activation token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_activation_key(user_id);
        conn.del(&key).await.map_err(|err: RedisError| {
            error!(error = %err, "Failed to delete activation token");
            format!("Failed to delete token: {}", err)
        })?;

        Ok(())
    }

    #[instrument(
        name = "redis.store_password_reset_token",
        skip(self, user_id, reset_token),
        fields(
            db.operation = "set",
            user.id = %user_id
        ),
        err
    )]
    async fn store_password_reset_token(
        &self,
        user_id: &str,
        reset_token: &str,
    ) -> Result<(), String> {
        debug!("Storing password reset token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_password_reset_key(user_id);
        conn.set_ex(&key, reset_token, *PASSWORD_RESET_TOKEN_TTL)
            .await
            .map_err(|err: RedisError| {
                error!(error = %err, "Failed to store password reset token");
                format!("Failed to store token: {}", err)
            })?;

        Ok(())
    }

    #[instrument(
        name = "redis.validate_password_reset_token",
        skip(self, user_id, reset_token),
        fields(
            db.operation = "get",
            user.id = %user_id
        ),
        err
    )]
    async fn validate_password_reset_token(
        &self,
        user_id: &str,
        reset_token: &str,
    ) -> Result<bool, String> {
        debug!("Validating password reset token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_password_reset_key(user_id);
        let stored_token: Option<String> = conn.get(&key).await.map_err(|err: RedisError| {
            error!(error = %err, "Failed to get password reset token");
            format!("Failed to get token: {}", err)
        })?;

        let is_valid = stored_token
            .as_ref()
            .map(|stored| stored.as_bytes().ct_eq(reset_token.as_bytes()).into())
            .unwrap_or(false);

        Ok(is_valid)
    }

    #[instrument(
        name = "redis.has_active_password_reset_token",
        skip(self, user_id),
        fields(
            db.operation = "exists",
            user.id = %user_id
        ),
        err
    )]
    async fn has_active_password_reset_token(&self, user_id: &str) -> Result<bool, String> {
        debug!("Checking for active password reset token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_password_reset_key(user_id);

        conn.exists(&key).await.map_err(|err: RedisError| {
            error!(error = %err, "Failed to check password reset token existence");
            format!("Failed to check token existence: {}", err)
        })
    }

    #[instrument(
        name = "redis.delete_password_reset_token",
        skip(self, user_id),
        fields(
            db.operation = "delete",
            user.id = %user_id
        ),
        err
    )]
    async fn delete_password_reset_token(&self, user_id: &str) -> Result<(), String> {
        debug!("Deleting password reset token");
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to connect to Redis");
                format!("Redis connection error: {}", err)
            })?;

        let key = self.user_password_reset_key(user_id);
        conn.del(&key).await.map_err(|err: RedisError| {
            error!(error = %err, "Failed to delete password reset token");
            format!("Failed to delete token: {}", err)
        })?;

        Ok(())
    }
}
