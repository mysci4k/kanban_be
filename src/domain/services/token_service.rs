use async_trait::async_trait;

#[async_trait]
pub trait TokenService: Send + Sync {
    async fn store_activation_token(
        &self,
        user_id: &str,
        activation_token: &str,
    ) -> Result<(), String>;
    async fn validate_activation_token(
        &self,
        user_id: &str,
        activation_token: &str,
    ) -> Result<bool, String>;
    async fn has_active_token(&self, user_id: &str) -> Result<bool, String>;
    async fn delete_activation_token(&self, user_id: &str) -> Result<(), String>;

    async fn store_password_reset_token(
        &self,
        user_id: &str,
        reset_token: &str,
    ) -> Result<(), String>;
    async fn validate_password_reset_token(
        &self,
        user_id: &str,
        reset_token: &str,
    ) -> Result<bool, String>;
    async fn has_active_password_reset_token(&self, user_id: &str) -> Result<bool, String>;
    async fn delete_password_reset_token(&self, user_id: &str) -> Result<(), String>;
}
