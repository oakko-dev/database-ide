use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::{connections::{ConnectionId, ConnectionInput, SavedConnection}, credentials::CredentialStore, error::AppError};

pub struct ConnectionService<C: CredentialStore, R: ConnectionRepository = InMemoryConnectionRepository> { pub credentials: C, pub repository: R }

pub trait ConnectionRepository {
    fn save(&self, connection: SavedConnection) -> Result<(), AppError>;
    fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError>;
    fn list(&self) -> Vec<SavedConnection>;
    fn delete(&self, id: ConnectionId) -> Result<(), AppError>;
}

#[derive(Default)]
pub struct InMemoryConnectionRepository(Mutex<HashMap<ConnectionId, SavedConnection>>);

impl ConnectionRepository for InMemoryConnectionRepository {
    fn save(&self, connection: SavedConnection) -> Result<(), AppError> { self.0.lock().unwrap().insert(connection.id, connection); Ok(()) }
    fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError> { self.0.lock().unwrap().get(&id).cloned().ok_or(AppError::NotFound) }
    fn list(&self) -> Vec<SavedConnection> { self.0.lock().unwrap().values().cloned().collect() }
    fn delete(&self, id: ConnectionId) -> Result<(), AppError> { self.0.lock().unwrap().remove(&id).map(|_| ()).ok_or(AppError::NotFound) }
}

impl<C: CredentialStore> ConnectionService<C, InMemoryConnectionRepository> {
    pub fn new(credentials: C) -> Self { Self { credentials, repository: InMemoryConnectionRepository::default() } }
}

impl<C: CredentialStore, R: ConnectionRepository> ConnectionService<C, R> {
    pub fn create(&self, input: ConnectionInput) -> Result<SavedConnection, AppError> {
        input.validate()?;
        let id = ConnectionId::new_v4();
        self.credentials.set_password(&id.to_string(), &input.password)?;
        let metadata = Self::metadata(id, &input);
        self.repository.save(metadata.clone())?;
        Ok(metadata)
    }
    pub fn update(&self, id: ConnectionId, input: ConnectionInput) -> Result<SavedConnection, AppError> {
        input.validate()?;
        self.credentials.set_password(&id.to_string(), &input.password)?;
        let metadata = Self::metadata(id, &input);
        self.repository.save(metadata.clone())?;
        Ok(metadata)
    }
    pub fn list(&self) -> Vec<SavedConnection> { self.repository.list() }
    pub fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError> { self.repository.get(id) }
    pub fn delete(&self, id: ConnectionId) -> Result<(), AppError> { self.repository.delete(id)?; self.credentials.delete_password(&id.to_string()) }
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
