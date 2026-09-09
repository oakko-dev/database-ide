use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

pub type ConnectionId = Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConnectionEnvironment {
    Local,
    Uat,
    Production,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionInput {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub environment: ConnectionEnvironment,
    pub read_only: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SavedConnection {
    pub id: ConnectionId,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub environment: ConnectionEnvironment,
    pub read_only: bool,
}

impl ConnectionInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::EmptyName);
        }
        if self.host.trim().is_empty() {
            return Err(AppError::EmptyHost);
        }
        if self.database.trim().is_empty() {
            return Err(AppError::EmptyDatabase);
        }
        if self.username.trim().is_empty() {
            return Err(AppError::EmptyUsername);
        }
        if self.port == 0 {
            return Err(AppError::InvalidPort);
        }
        Ok(())
    }

    pub fn validate_for_create(&self) -> Result<(), AppError> {
        self.validate()?;
        if self.password.is_empty() {
            return Err(AppError::EmptyPassword);
        }
        Ok(())
    }
}
