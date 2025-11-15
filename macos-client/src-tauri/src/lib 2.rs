use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[cfg(target_os = "macos")]
mod event_tap;
#[cfg(target_os = "macos")]
mod gestures;
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
    let mut app_state = state.lock().unwrap();

    match network::connect(&server_ip, port).await {
        Ok(client) => {
            app_state.network_client = Some(client);
            app_state.server_address = Some(format!("{}:{}", server_ip, port));
            Ok(format!("Connected to {}:{}", server_ip, port))
        }
        Err(e) => Err(format!("Failed to connect: {}", e)),
    }
}

/// Disconnect from Windows server
#[tauri::command]
async fn disconnect_from_server(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();
    app_state.network_client = None;
    app_state.server_address = None;
    Ok(())
}

/// Toggle input mode (macOS ↔ Windows)
#[tauri::command]
fn toggle_mode(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut app_state = state.lock().unwrap();

    app_state.mode = match app_state.mode {
        InputMode::MacOS => InputMode::Windows,
        InputMode::Windows => InputMode::MacOS,
    };

    Ok(format!("{:?}", app_state.mode))
}

/// Get current state
#[tauri::command]
fn get_state(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let app_state = state.lock().unwrap();
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
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::CGEventSource;

    // Try to create an event source - this will prompt for accessibility permissions if needed
    if CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::CombinedSessionState).is_err() {
        tracing::warn!("Accessibility permissions not granted. Please enable in System Settings.");
    }
}
