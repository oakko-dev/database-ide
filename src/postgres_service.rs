use postgres::{Client, NoTls};

use crate::{connections::{ConnectionId, ConnectionInput, SavedConnection}, credentials::CredentialStore, error::AppError};

pub struct ConnectionService<C: CredentialStore> { pub credentials: C }

impl<C: CredentialStore> ConnectionService<C> {
    pub fn new(credentials: C) -> Self { Self { credentials } }
    pub fn create(&self, input: ConnectionInput) -> Result<SavedConnection, AppError> {
        input.validate()?;
        let id = ConnectionId::new_v4();
        self.credentials.set_password(&id.to_string(), &input.password)?;
        Ok(Self::metadata(id, &input))
    }
    pub fn update(&self, id: ConnectionId, input: ConnectionInput) -> Result<SavedConnection, AppError> {
        input.validate()?;
        self.credentials.set_password(&id.to_string(), &input.password)?;
        Ok(Self::metadata(id, &input))
    }
    pub fn delete(&self, id: ConnectionId) -> Result<(), AppError> { self.credentials.delete_password(&id.to_string()) }
    pub fn test(&self, id: ConnectionId, metadata: &SavedConnection) -> Result<(), AppError> {
        let password = self.credentials.get_password(&id.to_string())?;
        let config = format!("host={} port={} dbname={} user={} password={}", metadata.host, metadata.port, metadata.database, metadata.username, password);
        let _client = Client::connect(&config, NoTls).map_err(|e| AppError::Database(Self::safe_db_error(&e.to_string())))?;
        Ok(())
    }
    fn metadata(id: ConnectionId, input: &ConnectionInput) -> SavedConnection { SavedConnection { id, name: input.name.clone(), host: input.host.clone(), port: input.port, database: input.database.clone(), username: input.username.clone(), environment: input.environment.clone(), read_only: input.read_only } }
    fn safe_db_error(message: &str) -> String { message.replace("password=", "password=[redacted]") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use crate::connections::ConnectionEnvironment;
    struct Fake(Mutex<HashMap<String, String>>);
    impl CredentialStore for Fake { fn set_password(&self, a: &str, p: &str) -> Result<(), AppError> { self.0.lock().unwrap().insert(a.into(), p.into()); Ok(()) } fn get_password(&self, a: &str) -> Result<String, AppError> { Ok(self.0.lock().unwrap().get(a).cloned().unwrap_or_default()) } fn delete_password(&self, a: &str) -> Result<(), AppError> { self.0.lock().unwrap().remove(a); Ok(()) } }
    #[test]
    fn create_stores_only_metadata_in_return_value() { let service = ConnectionService::new(Fake(Mutex::new(HashMap::new()))); let result = service.create(ConnectionInput { name: "Local".into(), host: "localhost".into(), port: 5432, database: "app".into(), username: "dev".into(), password: "secret".into(), environment: ConnectionEnvironment::Local, read_only: false }).unwrap(); assert_eq!(result.name, "Local"); }
    #[test]
    fn invalid_input_is_rejected() { let service = ConnectionService::new(Fake(Mutex::new(HashMap::new()))); let result = service.create(ConnectionInput { name: "".into(), host: "localhost".into(), port: 5432, database: "app".into(), username: "dev".into(), password: "secret".into(), environment: ConnectionEnvironment::Local, read_only: false }); assert!(matches!(result, Err(AppError::EmptyName))); }
}
