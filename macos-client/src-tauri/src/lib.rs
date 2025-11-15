use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[cfg(target_os = "macos")]
mod event_tap_simple;
#[cfg(target_os = "macos")]
use event_tap_simple as event_tap;

mod network;
mod state;

use state::{AppState, InputMode};

/// Connect to Windows server
#[tauri::command]
async fn connect_to_server(
    server_ip: String,
    port: u16,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let client = network::connect(&server_ip, port).await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let mut app_state = state.lock();
    app_state.network_client = Some(client);
    app_state.server_address = Some(format!("{}:{}", server_ip, port));

    Ok(format!("Connected to {}:{}", server_ip, port))
}

/// Disconnect from Windows server
#[tauri::command]
fn disconnect_from_server(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut app_state = state.lock();
    app_state.network_client = None;
    app_state.server_address = None;
    Ok(())
}

/// Toggle input mode (macOS ↔ Windows)
#[tauri::command]
fn toggle_mode(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut app_state = state.lock();

    app_state.mode = match app_state.mode {
        InputMode::MacOS => InputMode::Windows,
        InputMode::Windows => InputMode::MacOS,
    };

    Ok(format!("{:?}", app_state.mode))
}

/// Get current state
#[tauri::command]
fn get_state(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let app_state = state.lock();
    Ok(serde_json::to_string(&*app_state).map_err(|e| e.to_string())?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create shared application state
    let app_state = Arc::new(Mutex::new(AppState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            connect_to_server,
            disconnect_from_server,
            toggle_mode,
            get_state,
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                // Start event tap on macOS
                let app_handle = app.handle().clone();
                let state_clone = app_state.clone();

                std::thread::spawn(move || {
                    if let Err(e) = event_tap::start_event_tap(app_handle, state_clone) {
                        tracing::error!("Failed to start event tap: {}", e);
                    }
                });

                // Request accessibility permissions
                request_accessibility_permissions();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "macos")]
fn request_accessibility_permissions() {
    tracing::warn!("Please grant Accessibility permissions in System Settings → Privacy & Security → Accessibility");
    tracing::info!("This is required for the full event tap functionality");
}
