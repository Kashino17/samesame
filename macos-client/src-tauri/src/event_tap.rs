use anyhow::Result;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};
use samesame_protocol::{
    GestureEvent, GestureType, InputEvent, KeyboardEvent, Message, Modifiers, MouseButton,
    MouseButtonEvent, MouseMoveEvent, MouseScrollEvent,
};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tracing::{debug, error, info, warn};

use crate::state::{AppState, InputMode};

/// Start the event tap to capture keyboard and mouse events
pub fn start_event_tap(app_handle: AppHandle, state: Arc<Mutex<AppState>>) -> Result<()> {
    info!("Starting event tap...");

    // Event types we want to capture
    let event_mask = CGEventType::KeyDown as u64
        | CGEventType::KeyUp as u64
        | CGEventType::FlagsChanged as u64
        | CGEventType::LeftMouseDown as u64
        | CGEventType::LeftMouseUp as u64
        | CGEventType::RightMouseDown as u64
        | CGEventType::RightMouseUp as u64
        | CGEventType::MouseMoved as u64
        | CGEventType::LeftMouseDragged as u64
        | CGEventType::RightMouseDragged as u64
        | CGEventType::ScrollWheel as u64
        | CGEventType::OtherMouseDown as u64
        | CGEventType::OtherMouseUp as u64
        | CGEventType::OtherMouseDragged as u64;

    // Create the event tap callback
    let state_clone = state.clone();
    let callback = move |_proxy: CGEventTapProxy, event_type: CGEventType, event: &CGEvent| {
        match handle_event(event_type, event, &state_clone) {
            Ok(should_block) => {
                if should_block {
                    // Block the event (don't pass it to macOS)
                    None
                } else {
                    // Pass through the event
                    Some(event.to_owned())
                }
            }
            Err(e) => {
                error!("Error handling event: {}", e);
                Some(event.to_owned())
            }
        }
    };

    // Create event tap
    let event_tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        event_mask,
        callback,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to create event tap. Please grant accessibility permissions."))?;

    // Enable the event tap
    event_tap.enable();

    // Create a run loop source and add it to the current run loop
    let run_loop_source = event_tap
        .mach_port
        .create_runloop_source(0)
        .ok_or_else(|| anyhow::anyhow!("Failed to create run loop source"))?;

    let run_loop = CFRunLoop::get_current();
    run_loop.add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });

    info!("Event tap started successfully");

    // Run the event loop
    CFRunLoop::run_current();

    Ok(())
}

/// Handle a single event and decide whether to block it
fn handle_event(
    event_type: CGEventType,
    event: &CGEvent,
    state: &Arc<Mutex<AppState>>,
) -> Result<bool> {
    let mut app_state = state.lock().unwrap();

    // Check for Option+1 hotkey to toggle mode
    if is_toggle_hotkey(event_type, event) {
        app_state.mode = match app_state.mode {
            InputMode::MacOS => {
                info!("Switched to Windows mode");
                InputMode::Windows
            }
            InputMode::Windows => {
                info!("Switched to macOS mode");
                InputMode::MacOS
            }
        };
        // Don't block the toggle hotkey itself
        return Ok(false);
    }

    // If in macOS mode, pass through all events
    if app_state.mode == InputMode::MacOS {
        return Ok(false);
    }

    // If in Windows mode but not connected, still block to avoid duplicate input
    if app_state.network_client.is_none() {
        warn!("In Windows mode but not connected to server. Blocking input.");
        return Ok(true);
    }

    // Convert and forward the event
    if let Some(input_event) = convert_cg_event_to_input_event(event_type, event)? {
        let sequence = app_state.next_sequence();
        let message = Message::new(sequence, input_event);

        // Send to Windows server
        if let Some(ref mut client) = app_state.network_client {
            let client_clone = client.try_clone()?;
            let msg_clone = message.clone();

            // Send in background to avoid blocking event loop
            tokio::spawn(async move {
                let mut stream = client_clone;
                if let Err(e) = crate::network::send_event(&mut stream, msg_clone).await {
                    error!("Failed to send event: {}", e);
                }
            });
        }

        // Block the event so it doesn't go to macOS
        return Ok(true);
    }

    // If we couldn't convert the event, pass it through
    Ok(false)
}

/// Check if the event is the toggle hotkey (Option+1)
fn is_toggle_hotkey(event_type: CGEventType, event: &CGEvent) -> bool {
    if event_type != CGEventType::KeyDown {
        return false;
    }

    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    let flags = event.get_flags();

    // Option+1: keycode 18 is '1', Option flag is 0x80000 (524288)
    keycode == 18 && flags.contains(CGEventFlags::CGEventFlagAlternate)
}

/// Convert CGEvent to our InputEvent
fn convert_cg_event_to_input_event(
    event_type: CGEventType,
    event: &CGEvent,
) -> Result<Option<InputEvent>> {
    let result = match event_type {
        CGEventType::KeyDown | CGEventType::KeyUp => {
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
            let flags = event.get_flags();
            let pressed = event_type == CGEventType::KeyDown;

            // Try to get the character
            let mut character = None;
            if pressed {
                // Get Unicode string from event
                // This is a simplified version - full implementation would need more work
                let chars = event.get_string();
                if let Some(s) = chars {
                    character = s.chars().next();
                }
            }

            Some(InputEvent::Keyboard(KeyboardEvent {
                key_code: keycode,
                character,
                pressed,
                modifiers: extract_modifiers(flags),
            }))
        }

        CGEventType::MouseMoved | CGEventType::LeftMouseDragged | CGEventType::RightMouseDragged => {
            let location = event.location();
            let delta_x = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64;
            let delta_y = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64;

            // TODO: Normalize to 0.0-1.0 based on screen dimensions
            Some(InputEvent::MouseMove(MouseMoveEvent {
                x: location.x / 2560.0, // Simplified - should get actual screen width
                y: location.y / 1440.0, // Simplified - should get actual screen height
                delta_x,
                delta_y,
            }))
        }

        CGEventType::LeftMouseDown | CGEventType::LeftMouseUp => {
            Some(InputEvent::MouseButton(MouseButtonEvent {
                button: MouseButton::Left,
                pressed: event_type == CGEventType::LeftMouseDown,
            }))
        }

        CGEventType::RightMouseDown | CGEventType::RightMouseUp => {
            Some(InputEvent::MouseButton(MouseButtonEvent {
                button: MouseButton::Right,
                pressed: event_type == CGEventType::RightMouseDown,
            }))
        }

        CGEventType::OtherMouseDown | CGEventType::OtherMouseUp => {
            let button_number = event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER);
            let button = match button_number {
                2 => MouseButton::Middle,
                3 => MouseButton::Button4,
                4 => MouseButton::Button5,
                _ => return Ok(None),
            };

            Some(InputEvent::MouseButton(MouseButtonEvent {
                button,
                pressed: event_type == CGEventType::OtherMouseDown,
            }))
        }

        CGEventType::ScrollWheel => {
            let delta_y = event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1) as f64;
            let delta_x = event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2) as f64;
            let is_pixel_based = event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_IS_CONTINUOUS) != 0;

            Some(InputEvent::MouseScroll(MouseScrollEvent {
                delta_x,
                delta_y,
                is_pixel_based,
            }))
        }

        _ => None,
    };

    Ok(result)
}

/// Extract modifier keys from CGEventFlags
fn extract_modifiers(flags: CGEventFlags) -> Modifiers {
    Modifiers {
        shift: flags.contains(CGEventFlags::CGEventFlagShift),
        control: flags.contains(CGEventFlags::CGEventFlagControl),
        alt: flags.contains(CGEventFlags::CGEventFlagAlternate),
        command: flags.contains(CGEventFlags::CGEventFlagCommand),
    }
}

// Placeholder type for CGEventTapProxy
type CGEventTapProxy = u32;
