// Simplified gesture handling
// This is a placeholder - full implementation would use NSEvent monitoring

use parking_lot::Mutex;
use std::sync::Arc;
use tracing::info;

use crate::state::AppState;

/// Start gesture monitor (simplified version)
#[allow(dead_code)]
pub fn start_gesture_monitor(_state: Arc<Mutex<AppState>>) {
    info!("Gesture monitor starting (simplified version)...");
    info!("Full gesture support requires additional work with NSEvent");
}
