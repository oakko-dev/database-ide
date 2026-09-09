use postgres::{Client, Config, NoTls};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::{
    connections::{ConnectionId, ConnectionInput, SavedConnection},
    credentials::CredentialStore,
    error::AppError,
};

pub struct ConnectionService<
    C: CredentialStore,
    R: ConnectionRepository = InMemoryConnectionRepository,
> {
    pub credentials: C,
    pub repository: R,
}

pub trait ConnectionRepository {
    fn save(&self, connection: SavedConnection) -> Result<(), AppError>;
    fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError>;
    fn list(&self) -> Vec<SavedConnection>;
    fn delete(&self, id: ConnectionId) -> Result<(), AppError>;
}

#[derive(Default)]
pub struct InMemoryConnectionRepository(Mutex<HashMap<ConnectionId, SavedConnection>>);

impl ConnectionRepository for InMemoryConnectionRepository {
    fn save(&self, connection: SavedConnection) -> Result<(), AppError> {
        self.0.lock().unwrap().insert(connection.id, connection);
        Ok(())
    }
    fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError> {
        self.0
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound)
    }
    fn list(&self) -> Vec<SavedConnection> {
        self.0.lock().unwrap().values().cloned().collect()
    }
    fn delete(&self, id: ConnectionId) -> Result<(), AppError> {
        self.0
            .lock()
            .unwrap()
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
    }
}

pub struct FileConnectionRepository {
    path: PathBuf,
    state: Mutex<HashMap<ConnectionId, SavedConnection>>,
}

impl FileConnectionRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref().to_path_buf();
        let state = if path.exists() {
            let contents = fs::read_to_string(&path).map_err(|e| {
                AppError::Database(format!("could not read saved connections: {e}"))
            })?;
            serde_json::from_str(&contents)
                .map_err(|e| AppError::Database(format!("could not read saved connections: {e}")))?
        } else {
            HashMap::new()
        };
        Ok(Self {
            path,
            state: Mutex::new(state),
        })
    }

    fn flush(&self) -> Result<(), AppError> {
        let state = self.state.lock().unwrap();
        let contents = serde_json::to_string_pretty(&*state)
            .map_err(|e| AppError::Database(format!("could not save connections: {e}")))?;
        fs::write(&self.path, contents)
            .map_err(|e| AppError::Database(format!("could not save connections: {e}")))
    }
}

impl ConnectionRepository for FileConnectionRepository {
    fn save(&self, connection: SavedConnection) -> Result<(), AppError> {
        self.state.lock().unwrap().insert(connection.id, connection);
        self.flush()
    }
    fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError> {
        self.state
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound)
    }
    fn list(&self) -> Vec<SavedConnection> {
        self.state.lock().unwrap().values().cloned().collect()
    }
    fn delete(&self, id: ConnectionId) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
            .and_then(|_| self.flush())
    }
}

impl<C: CredentialStore> ConnectionService<C, InMemoryConnectionRepository> {
    pub fn new(credentials: C) -> Self {
        Self {
            credentials,
            repository: InMemoryConnectionRepository::default(),
        }
    }
}

impl<C: CredentialStore, R: ConnectionRepository> ConnectionService<C, R> {
    pub fn with_repository(credentials: C, repository: R) -> Self {
        Self {
            credentials,
            repository,
        }
    }
}

impl<C: CredentialStore, R: ConnectionRepository> ConnectionService<C, R> {
    pub fn create(&self, input: ConnectionInput) -> Result<SavedConnection, AppError> {
        input.validate_for_create()?;
        let id = ConnectionId::new_v4();
        self.credentials
            .set_password(&id.to_string(), &input.password)?;
        let metadata = Self::metadata(id, &input);
        self.repository.save(metadata.clone())?;
        Ok(metadata)
    }
    pub fn update(
        &self,
        id: ConnectionId,
        input: ConnectionInput,
    ) -> Result<SavedConnection, AppError> {
        input.validate()?;
        self.repository.get(id)?;
        if !input.password.is_empty() {
            self.credentials
                .set_password(&id.to_string(), &input.password)?;
        }
        let metadata = Self::metadata(id, &input);
        self.repository.save(metadata.clone())?;
        Ok(metadata)
    }
    pub fn list(&self) -> Vec<SavedConnection> {
        self.repository.list()
    }
    pub fn get(&self, id: ConnectionId) -> Result<SavedConnection, AppError> {
        self.repository.get(id)
    }
    pub fn delete(&self, id: ConnectionId) -> Result<(), AppError> {
        self.repository.delete(id)?;
        self.credentials.delete_password(&id.to_string())
    }
    pub fn test(&self, id: ConnectionId, metadata: &SavedConnection) -> Result<(), AppError> {
        let password = self.credentials.get_password(&id.to_string())?;
        let mut config = Config::new();
        config
            .host(&metadata.host)
            .port(metadata.port)
            .dbname(&metadata.database)
            .user(&metadata.username)
            .password(password);
        if metadata.read_only {
            config.options("-c default_transaction_read_only=on");
        }
        let _client = config
            .connect(NoTls)
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
    pub fn with_client<T>(
        &self,
        id: ConnectionId,
        operation: impl FnOnce(&mut Client) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let metadata = self.repository.get(id)?;
        let password = self.credentials.get_password(&id.to_string())?;
        let mut config = Config::new();
        config
            .host(&metadata.host)
            .port(metadata.port)
            .dbname(&metadata.database)
            .user(&metadata.username)
            .password(password);
        if metadata.read_only {
            config.options("-c default_transaction_read_only=on");
        }
        let mut client = config
            .connect(NoTls)
            .map_err(|e| AppError::Database(format!("database connection failed: {e}")))?;
        operation(&mut client)
    }
    fn metadata(id: ConnectionId, input: &ConnectionInput) -> SavedConnection {
        SavedConnection {
            id,
            name: input.name.clone(),
            host: input.host.clone(),
            port: input.port,
            database: input.database.clone(),
            username: input.username.clone(),
            environment: input.environment.clone(),
            read_only: input.read_only,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connections::ConnectionEnvironment;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    struct Fake(Arc<Mutex<HashMap<String, String>>>);
    impl CredentialStore for Fake {
        fn set_password(&self, a: &str, p: &str) -> Result<(), AppError> {
            self.0.lock().unwrap().insert(a.into(), p.into());
            Ok(())
        }
        fn get_password(&self, a: &str) -> Result<String, AppError> {
            Ok(self.0.lock().unwrap().get(a).cloned().unwrap_or_default())
        }
        fn delete_password(&self, a: &str) -> Result<(), AppError> {
            self.0.lock().unwrap().remove(a);
            Ok(())
        }
    }
    fn fake() -> (Fake, Arc<Mutex<HashMap<String, String>>>) {
        let passwords = Arc::new(Mutex::new(HashMap::new()));
        (Fake(passwords.clone()), passwords)
    }
    fn input(password: &str) -> ConnectionInput {
        ConnectionInput {
            name: "Local".into(),
            host: "localhost".into(),
            port: 5432,
            database: "app".into(),
            username: "dev".into(),
            password: password.into(),
            environment: ConnectionEnvironment::Local,
            read_only: false,
        }
    }
    #[test]
    fn create_stores_only_metadata_in_return_value() {
        let (fake, passwords) = fake();
        let service = ConnectionService::new(fake);
        let result = service.create(input("secret")).unwrap();
        assert_eq!(result.name, "Local");
        assert!(!serde_json::to_string(&result).unwrap().contains("secret"));
        assert_eq!(passwords.lock().unwrap().len(), 1);
    }
    #[test]
    fn invalid_input_is_rejected() {
        let (fake, _) = fake();
        let service = ConnectionService::new(fake);
        let mut invalid = input("secret");
        invalid.name.clear();
        let result = service.create(invalid);
        assert!(matches!(result, Err(AppError::EmptyName)));
    }
    #[test]
    fn create_requires_a_password() {
        let (fake, _) = fake();
        let service = ConnectionService::new(fake);
        let result = service.create(input(""));
        assert!(matches!(result, Err(AppError::EmptyPassword)));
    }
    #[test]
    fn update_without_password_preserves_existing_credential() {
        let (fake, passwords) = fake();
        let service = ConnectionService::new(fake);
        let saved = service.create(input("secret")).unwrap();
        let mut edited = input("");
        edited.name = "Renamed".into();
        service.update(saved.id, edited).unwrap();
        assert_eq!(
            passwords.lock().unwrap().get(&saved.id.to_string()),
            Some(&"secret".to_string())
        );
    }
    #[test]
    fn file_repository_persists_metadata_without_password() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("connections.json");
        let (fake, _) = fake();
        let service = ConnectionService::with_repository(
            fake,
            FileConnectionRepository::open(&path).unwrap(),
        );
        service.create(input("secret")).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(!contents.contains("secret"));
        assert_eq!(
            FileConnectionRepository::open(&path).unwrap().list().len(),
            1
        );
    }
    #[test]
    fn update_requires_an_existing_connection() {
        let (fake, _) = fake();
        let service = ConnectionService::new(fake);
        let result = service.update(ConnectionId::new_v4(), input("secret"));
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
