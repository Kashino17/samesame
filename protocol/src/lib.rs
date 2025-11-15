use serde::{Deserialize, Serialize};

/// All event types that can be sent from macOS to Windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard key press/release
    Keyboard(KeyboardEvent),
    /// Mouse movement
    MouseMove(MouseMoveEvent),
    /// Mouse button press/release
    MouseButton(MouseButtonEvent),
    /// Mouse scroll (2-finger trackpad scroll)
    MouseScroll(MouseScrollEvent),
    /// Trackpad gesture (multi-finger swipes, etc.)
    Gesture(GestureEvent),
    /// Ping to check connection
    Ping,
    /// Pong response
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    /// Virtual key code
    pub key_code: u16,
    /// Character (if applicable)
    pub character: Option<char>,
    /// Is key pressed (true) or released (false)
    pub pressed: bool,
    /// Modifier keys state
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveEvent {
    /// Absolute X position (0.0 to 1.0 normalized)
    pub x: f64,
    /// Absolute Y position (0.0 to 1.0 normalized)
    pub y: f64,
    /// Relative X movement
    pub delta_x: f64,
    /// Relative Y movement
    pub delta_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub pressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Button4,
    Button5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseScrollEvent {
    /// Horizontal scroll delta (negative = left, positive = right)
    pub delta_x: f64,
    /// Vertical scroll delta (negative = down, positive = up)
    /// Note: macOS natural scrolling is inverted compared to Windows
    pub delta_y: f64,
    /// Is this a pixel-based scroll (trackpad) or line-based (mouse wheel)
    pub is_pixel_based: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEvent {
    pub gesture_type: GestureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureType {
    /// 4-finger swipe left (switch desktop left on Windows)
    SwipeLeft,
    /// 4-finger swipe right (switch desktop right on Windows)
    SwipeRight,
    /// 4-finger swipe up (Task View on Windows)
    SwipeUp,
    /// 4-finger swipe down (Show Desktop on Windows)
    SwipeDown,
    /// Pinch to zoom in
    ZoomIn(f64),
    /// Pinch to zoom out
    ZoomOut(f64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,  // macOS Control key
    pub alt: bool,      // macOS Option/Alt key
    pub command: bool,  // macOS Command key
}

impl Modifiers {
    /// Convert macOS modifiers to Windows modifiers
    /// Command -> Ctrl, Option -> Alt, Control -> Win
    pub fn to_windows(&self) -> WindowsModifiers {
        WindowsModifiers {
            shift: self.shift,
            ctrl: self.command,      // macOS Cmd → Windows Ctrl
            alt: self.alt,           // macOS Option → Windows Alt
            win: self.control,       // macOS Ctrl → Windows Win (configurable)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct WindowsModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

/// Message frame for TCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message sequence number for ordering
    pub sequence: u64,
    /// The actual event
    pub event: InputEvent,
}

impl Message {
    pub fn new(sequence: u64, event: InputEvent) -> Self {
        Self { sequence, event }
    }

    /// Serialize message to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes using bincode
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}
