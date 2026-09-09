use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("connection name must not be empty")]
    EmptyName,
    #[error("host must not be empty")]
    EmptyHost,
    #[error("database must not be empty")]
    EmptyDatabase,
    #[error("username must not be empty")]
    EmptyUsername,
    #[error("password must not be empty when creating a connection")]
    EmptyPassword,
    #[error("port must be between 1 and 65535")]
    InvalidPort,
    #[error("saved connection was not found")]
    NotFound,
    #[error("credential storage error: {0}")]
    CredentialStorage(String),
    #[error("database connection failed: {0}")]
    Database(String),
}
