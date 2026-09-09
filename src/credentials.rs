use keyring::Entry;

use crate::error::AppError;

const SERVICE: &str = "com.oakko.database-ide.postgres";

pub trait CredentialStore {
    fn set_password(&self, account: &str, password: &str) -> Result<(), AppError>;
    fn get_password(&self, account: &str) -> Result<String, AppError>;
    fn delete_password(&self, account: &str) -> Result<(), AppError>;
}

#[derive(Default)]
pub struct OsCredentialStore;

impl OsCredentialStore {
    fn entry(account: &str) -> Result<Entry, AppError> {
        Entry::new(SERVICE, account).map_err(|e| AppError::CredentialStorage(e.to_string()))
    }
}

impl CredentialStore for OsCredentialStore {
    fn set_password(&self, account: &str, password: &str) -> Result<(), AppError> {
        Self::entry(account)?.set_password(password).map_err(|e| AppError::CredentialStorage(e.to_string()))
    }
    fn get_password(&self, account: &str) -> Result<String, AppError> {
        Self::entry(account)?.get_password().map_err(|e| AppError::CredentialStorage(e.to_string()))
    }
    fn delete_password(&self, account: &str) -> Result<(), AppError> {
        Self::entry(account)?.delete_credential().map_err(|e| AppError::CredentialStorage(e.to_string()))
    }
}
