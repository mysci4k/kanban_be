use crate::{
    domain::services::{EmailService, EmailTemplate},
    shared::utils::constants::{BASE_URL, FROM_EMAIL, SMTP_PASSWORD, SMTP_SERVER, SMTP_USERNAME},
};
use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tera::{Context, Tera};

pub struct SmtpEmailService {
    smtp_transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    base_url: String,
    tera: Tera,
}

impl SmtpEmailService {
    pub fn new() -> Result<Self, String> {
        let from_email = FROM_EMAIL.clone();
        let base_url = BASE_URL.clone();

        let credentials = Credentials::new(SMTP_USERNAME.clone(), SMTP_PASSWORD.clone());

        let smtp_transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&SMTP_SERVER)
            .map_err(|err| format!("Failed to create SMTP transport: {}", err))?
            .credentials(credentials)
            .build();

        let tera = Tera::new("templates/emails/**/*")
            .map_err(|err| format!("Failed to initialize Tera: {}", err))?;

        Ok(SmtpEmailService {
            smtp_transport,
            from_email,
            base_url,
            tera,
        })
    }

    fn build_context(&self, template: &EmailTemplate) -> Context {
        let mut context = Context::new();

        match template {
            EmailTemplate::Activation {
                username,
                activation_link,
            } => {
                context.insert("username", username);
                context.insert("activation_link", activation_link);
            }
            EmailTemplate::PasswordReset {
                username,
                reset_link,
            } => {
                context.insert("username", username);
                context.insert("reset_link", reset_link);
            }
        }

        context
    }

    async fn send_templated_email(
        &self,
        to_email: &str,
        subject: &str,
        html_template: &str,
        text_template: &str,
        context: &Context,
    ) -> Result<(), String> {
        let html_body = self
            .tera
            .render(html_template, context)
            .map_err(|err| format!("Failed to render HTML template: {}", err))?;

        let text_body = self
            .tera
            .render(text_template, context)
            .map_err(|err| format!("Failed to render text template: {}", err))?;

        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|err| format!("Invalid from email: {}", err))?,
            )
            .to(to_email
                .parse()
                .map_err(|err| format!("Invalid to email: {}", err))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )
            .map_err(|err| format!("Failed to build email: {}", err))?;

        self.smtp_transport
            .send(email)
            .await
            .map_err(|err| format!("Failed to send email: {}", err))?;

        Ok(())
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_email(&self, to_email: &str, template: EmailTemplate) -> Result<(), String> {
        let context = self.build_context(&template);
        let (html_template, text_template) = template.template_name();
        let subject = template.subject();

        self.send_templated_email(to_email, subject, html_template, text_template, &context)
            .await
    }

    async fn send_activation_email(
        &self,
        to_email: &str,
        username: &str,
        user_id: &str,
        activation_token: &str,
    ) -> Result<(), String> {
        let activation_link = format!(
            "{}/api/activate?userId={}?token={}",
            self.base_url, user_id, activation_token
        );

        let template = EmailTemplate::Activation {
            username: username.to_string(),
            activation_link,
        };

        self.send_email(to_email, template).await
    }

    async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        user_id: &str,
        reset_token: &str,
    ) -> Result<(), String> {
        let reset_link = format!(
            "{}/api/reset-password?userId={}&token={}",
            self.base_url, user_id, reset_token
        );

        let template = EmailTemplate::PasswordReset {
            username: username.to_string(),
            reset_link,
        };

        self.send_email(to_email, template).await
    }
}
