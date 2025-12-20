use async_trait::async_trait;

#[derive(Debug)]
pub enum EmailTemplate {
    Activation {
        username: String,
        activation_link: String,
    },
    PasswordReset {
        username: String,
        reset_link: String,
    },
}

impl EmailTemplate {
    pub fn template_name(&self) -> (&str, &str) {
        match self {
            EmailTemplate::Activation { .. } => (
                "activation/html_template.html",
                "activation/text_template.txt",
            ),
            EmailTemplate::PasswordReset { .. } => (
                "password_reset/html_template.html",
                "password_reset/text_template.txt",
            ),
        }
    }

    pub fn subject(&self) -> &str {
        match self {
            EmailTemplate::Activation { .. } => "Activate your Kanblast account",
            EmailTemplate::PasswordReset { .. } => "Reset your Kanblast password",
        }
    }
}

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(&self, to_email: &str, template: EmailTemplate) -> Result<(), String>;
    async fn send_activation_email(
        &self,
        to_email: &str,
        username: &str,
        user_id: &str,
        activation_token: &str,
    ) -> Result<(), String>;
    async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        user_id: &str,
        reset_token: &str,
    ) -> Result<(), String>;
}
