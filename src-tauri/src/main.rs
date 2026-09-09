#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use database_ide::{
    connections::{ConnectionId, ConnectionInput, SavedConnection},
    credentials::OsCredentialStore,
    postgres_service::{ConnectionService, FileConnectionRepository},
    AppError,
};
use tauri::{AppHandle, Manager, State};

type AppService = Mutex<ConnectionService<OsCredentialStore, FileConnectionRepository>>;

#[derive(serde::Deserialize)]
struct UpdateRequest { id: ConnectionId, input: ConnectionInput }

fn safe_error(error: AppError) -> String { error.to_string() }

#[tauri::command]
fn list_connections(service: State<'_, AppService>) -> Result<Vec<SavedConnection>, String> {
    Ok(service.lock().map_err(|_| "connection service is unavailable".to_string())?.list())
}

#[tauri::command]
fn create_connection(service: State<'_, AppService>, input: ConnectionInput) -> Result<SavedConnection, String> {
    service.lock().map_err(|_| "connection service is unavailable".to_string())?.create(input).map_err(safe_error)
}

#[tauri::command]
fn update_connection(service: State<'_, AppService>, request: UpdateRequest) -> Result<SavedConnection, String> {
    service.lock().map_err(|_| "connection service is unavailable".to_string())?.update(request.id, request.input).map_err(safe_error)
}

#[tauri::command]
fn delete_connection(service: State<'_, AppService>, id: ConnectionId) -> Result<(), String> {
    service.lock().map_err(|_| "connection service is unavailable".to_string())?.delete(id).map_err(safe_error)
}

#[tauri::command]
fn test_connection(service: State<'_, AppService>, id: ConnectionId) -> Result<(), String> {
    let guard = service.lock().map_err(|_| "connection service is unavailable".to_string())?;
    let metadata = guard.get(id).map_err(safe_error)?;
    guard.test(id, &metadata).map_err(safe_error)
}

fn build_service(app: &AppHandle) -> Result<AppService, Box<dyn std::error::Error>> {
    let directory = app.path().app_data_dir()?;
    std::fs::create_dir_all(&directory)?;
    let repository = FileConnectionRepository::open(directory.join("connections.json"))?;
    Ok(Mutex::new(ConnectionService::with_repository(OsCredentialStore, repository)))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| { app.manage(build_service(app.handle())?); Ok(()) })
        .invoke_handler(tauri::generate_handler![list_connections, create_connection, update_connection, delete_connection, test_connection])
        .run(tauri::generate_context!())
        .expect("error while running Database IDE");
}
