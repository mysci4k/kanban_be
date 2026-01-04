use regex::Regex;
use std::{env, sync::LazyLock};

// Server configuration constants
pub static SERVER_ADDRESS: LazyLock<String> =
    LazyLock::new(|| env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1".to_string()));

pub static SERVER_PORT: LazyLock<u16> = LazyLock::new(|| {
    env::var("SERVER_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid u16 number")
});

// Database configuration constants
pub static DATABASE_URL: LazyLock<String> =
    LazyLock::new(|| env::var("DATABASE_URL").expect("Missing DATABASE_URL environment variable"));

pub static REDIS_URL: LazyLock<String> =
    LazyLock::new(|| env::var("REDIS_URL").expect("Missing REDIS_URL environment variable"));

// Authentication and security configuration constants
pub static SESSION_KEY: LazyLock<String> =
    LazyLock::new(|| env::var("SESSION_KEY").expect("Missing SESSION_KEY environment variable"));

pub static ACTIVATION_TOKEN_TTL: LazyLock<u64> = LazyLock::new(|| {
    env::var("ACTIVATION_TOKEN_TTL")
        .unwrap_or("3600".to_string())
        .parse()
        .expect("ACTIVATION_TOKEN_TTL must be a valid u64 number")
});

pub static PASSWORD_RESET_TOKEN_TTL: LazyLock<u64> = LazyLock::new(|| {
    env::var("PASSWORD_RESET_TOKEN_TTL")
        .unwrap_or("3600".to_string())
        .parse()
        .expect("PASSWORD_RESET_TOKEN_TTL must be a valid u64 number")
});

// Email configuration constants
pub static SMTP_SERVER: LazyLock<String> =
    LazyLock::new(|| env::var("SMTP_SERVER").expect("Missing SMTP_SERVER environment variable"));

pub static SMTP_USERNAME: LazyLock<String> = LazyLock::new(|| {
    env::var("SMTP_USERNAME").expect("Missing SMTP_USERNAME environment variable")
});

pub static SMTP_PASSWORD: LazyLock<String> = LazyLock::new(|| {
    env::var("SMTP_PASSWORD").expect("Missing SMTP_PASSWORD environment variable")
});

pub static FROM_EMAIL: LazyLock<String> =
    LazyLock::new(|| env::var("FROM_EMAIL").expect("Missing FROM_EMAIL environment variable"));

pub static BASE_URL: LazyLock<String> =
    LazyLock::new(|| env::var("BASE_URL").expect("Missing BASE_URL environment variable"));

// Tracing configuration constants
pub static OTLP_ENDPOINT: LazyLock<Option<String>> =
    LazyLock::new(|| env::var("OTLP_ENDPOINT").ok());

pub static OTLP_SERVICE_NAME: LazyLock<String> =
    LazyLock::new(|| env::var("OTLP_SERVICE_NAME").unwrap_or("kanban_api".to_string()));

pub static OTLP_SAMPLING_RATIO: LazyLock<f64> = LazyLock::new(|| {
    env::var("OTLP_SAMPLING_RATIO")
        .unwrap_or("1.0".to_string())
        .parse()
        .expect("SAMPLING_RATIO must be a valid f64 number")
});

// Regular expressions for validation
pub static RE_ONLY_LETTERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{L}+$").unwrap());
