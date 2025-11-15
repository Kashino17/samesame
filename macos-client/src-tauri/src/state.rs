use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    /// Inputs go to macOS (normal mode)
    MacOS,
    /// Inputs are forwarded to Windows (forwarding mode)
    Windows,
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    /// Current input mode
    pub mode: InputMode,
    /// TCP connection to Windows server (if connected)
    #[serde(skip)]
    pub network_client: Option<TcpStream>,
    /// Server address (IP:PORT)
    pub server_address: Option<String>,
    /// Message sequence number
    pub sequence: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: InputMode::MacOS,
            network_client: None,
            server_address: None,
            sequence: 0,
        }
    }

    /// Get next sequence number
    pub fn next_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }

    /// Check if we should forward inputs
    pub fn should_forward(&self) -> bool {
        self.mode == InputMode::Windows && self.network_client.is_some()
    }
}
