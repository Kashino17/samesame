use anyhow::{Result, anyhow};
use samesame_protocol::{
    InputEvent, KeyboardEvent, MouseMoveEvent, MouseButtonEvent, MouseScrollEvent,
    GestureEvent, GestureType, MouseButton,
};
use std::mem;
use tracing::{debug, warn};

#[cfg(windows)]
use windows::Win32::{
    Foundation::POINT,
    UI::Input::KeyboardAndMouse::*,
    UI::WindowsAndMessaging::*,
};

/// Simulate an input event on Windows
pub fn simulate_event(event: &InputEvent) -> Result<()> {
    match event {
        InputEvent::Keyboard(kb_event) => simulate_keyboard(kb_event),
        InputEvent::MouseMove(move_event) => simulate_mouse_move(move_event),
        InputEvent::MouseButton(btn_event) => simulate_mouse_button(btn_event),
        InputEvent::MouseScroll(scroll_event) => simulate_mouse_scroll(scroll_event),
        InputEvent::Gesture(gesture_event) => simulate_gesture(gesture_event),
        InputEvent::Ping | InputEvent::Pong => Ok(()),
    }
}

#[cfg(windows)]
fn simulate_keyboard(event: &KeyboardEvent) -> Result<()> {
    let mut inputs = Vec::new();

    // Map macOS modifiers to Windows modifiers
    let win_mods = event.modifiers.to_windows();

    // Press modifiers first
    if win_mods.ctrl {
        inputs.push(create_key_input(VK_CONTROL.0 as u16, true, false));
    }
    if win_mods.shift {
        inputs.push(create_key_input(VK_SHIFT.0 as u16, true, false));
    }
    if win_mods.alt {
        inputs.push(create_key_input(VK_MENU.0 as u16, true, false));
    }
    if win_mods.win {
        inputs.push(create_key_input(VK_LWIN.0 as u16, true, false));
    }

    // Map the actual key
    let vk_code = map_macos_to_windows_key(event.key_code);

    if event.pressed {
        inputs.push(create_key_input(vk_code, true, false));
    } else {
        inputs.push(create_key_input(vk_code, false, false));
    }

    // Release modifiers after key (in reverse order)
    if !event.pressed {
        if win_mods.win {
            inputs.push(create_key_input(VK_LWIN.0 as u16, false, false));
        }
        if win_mods.alt {
            inputs.push(create_key_input(VK_MENU.0 as u16, false, false));
        }
        if win_mods.shift {
            inputs.push(create_key_input(VK_SHIFT.0 as u16, false, false));
        }
        if win_mods.ctrl {
            inputs.push(create_key_input(VK_CONTROL.0 as u16, false, false));
        }
    }

    send_inputs(&inputs)?;

    debug!(
        "Simulated keyboard: key_code={}, pressed={}, char={:?}",
        vk_code, event.pressed, event.character
    );

    Ok(())
}

#[cfg(windows)]
fn create_key_input(vk_code: u16, is_press: bool, is_unicode: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if !is_press {
        flags |= KEYEVENTF_KEYUP;
    }
    if is_unicode {
        flags |= KEYEVENTF_UNICODE;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk_code),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(windows)]
fn simulate_mouse_move(event: &MouseMoveEvent) -> Result<()> {
    unsafe {
        // Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // Convert normalized coordinates (0.0-1.0) to absolute screen coordinates
        let abs_x = (event.x * 65535.0) as i32;
        let abs_y = (event.y * 65535.0) as i32;

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: abs_x,
                    dy: abs_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        send_inputs(&[input])?;
    }

    Ok(())
}

#[cfg(windows)]
fn simulate_mouse_button(event: &MouseButtonEvent) -> Result<()> {
    let flags = match (&event.button, event.pressed) {
        (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
        (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
        (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
        (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
        (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
        (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
        (MouseButton::Button4, true) => MOUSEEVENTF_XDOWN,
        (MouseButton::Button4, false) => MOUSEEVENTF_XUP,
        (MouseButton::Button5, true) => MOUSEEVENTF_XDOWN,
        (MouseButton::Button5, false) => MOUSEEVENTF_XUP,
    };

    let mouse_data = match event.button {
        MouseButton::Button4 => 0x0001, // XBUTTON1
        MouseButton::Button5 => 0x0002, // XBUTTON2
        _ => 0,
    };

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    send_inputs(&[input])?;

    debug!(
        "Simulated mouse button: {:?}, pressed={}",
        event.button, event.pressed
    );

    Ok(())
}

#[cfg(windows)]
fn simulate_mouse_scroll(event: &MouseScrollEvent) -> Result<()> {
    // Windows expects scroll in units of WHEEL_DELTA (120)
    // macOS trackpad gives pixel-based scrolling, we need to convert
    let delta_multiplier = if event.is_pixel_based { 1.0 } else { 120.0 };

    // Vertical scroll (invert for Windows - macOS natural scrolling is opposite)
    if event.delta_y.abs() > 0.01 {
        let delta = (-event.delta_y * delta_multiplier) as i32;
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: delta as u32,
                    dwFlags: MOUSEEVENTF_WHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        send_inputs(&[input])?;
    }

    // Horizontal scroll
    if event.delta_x.abs() > 0.01 {
        let delta = (event.delta_x * delta_multiplier) as i32;
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: delta as u32,
                    dwFlags: MOUSEEVENTF_HWHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        send_inputs(&[input])?;
    }

    Ok(())
}

#[cfg(windows)]
fn simulate_gesture(event: &GestureEvent) -> Result<()> {
    // Map macOS gestures to Windows shortcuts
    match event.gesture_type {
        GestureType::SwipeLeft => {
            // Windows: Ctrl+Win+Left (switch virtual desktop left)
            simulate_key_combo(&[VK_LCONTROL.0 as u16, VK_LWIN.0 as u16, VK_LEFT.0 as u16])?;
        }
        GestureType::SwipeRight => {
            // Windows: Ctrl+Win+Right (switch virtual desktop right)
            simulate_key_combo(&[VK_LCONTROL.0 as u16, VK_LWIN.0 as u16, VK_RIGHT.0 as u16])?;
        }
        GestureType::SwipeUp => {
            // Windows: Win+Tab (Task View)
            simulate_key_combo(&[VK_LWIN.0 as u16, VK_TAB.0 as u16])?;
        }
        GestureType::SwipeDown => {
            // Windows: Win+D (Show Desktop)
            simulate_key_combo(&[VK_LWIN.0 as u16, 0x44])?; // 'D' key
        }
        GestureType::ZoomIn(_) | GestureType::ZoomOut(_) => {
            // Could be mapped to Ctrl+Plus/Minus in the future
            warn!("Zoom gestures not yet implemented");
        }
    }

    debug!("Simulated gesture: {:?}", event.gesture_type);

    Ok(())
}

#[cfg(windows)]
fn simulate_key_combo(keys: &[u16]) -> Result<()> {
    let mut inputs = Vec::new();

    // Press all keys
    for &key in keys {
        inputs.push(create_key_input(key, true, false));
    }

    // Release all keys in reverse order
    for &key in keys.iter().rev() {
        inputs.push(create_key_input(key, false, false));
    }

    send_inputs(&inputs)?;

    Ok(())
}

#[cfg(windows)]
fn send_inputs(inputs: &[INPUT]) -> Result<()> {
    unsafe {
        let sent = SendInput(
            inputs,
            mem::size_of::<INPUT>() as i32,
        );

        if sent as usize != inputs.len() {
            return Err(anyhow!(
                "Failed to send all inputs. Sent {} out of {}",
                sent,
                inputs.len()
            ));
        }
    }

    Ok(())
}

/// Map macOS virtual key codes to Windows virtual key codes
/// This is a simplified mapping - full mapping would need a comprehensive table
#[cfg(windows)]
fn map_macos_to_windows_key(macos_key: u16) -> u16 {
    // macOS key codes: https://eastmanreference.com/complete-list-of-applescript-key-codes
    match macos_key {
        // Letters A-Z (macOS: 0-11, 12-35)
        0x00 => 0x41,  // A
        0x0B => 0x42,  // B
        0x08 => 0x43,  // C
        0x02 => 0x44,  // D
        0x0E => 0x45,  // E
        0x03 => 0x46,  // F
        0x05 => 0x47,  // G
        0x04 => 0x48,  // H
        0x22 => 0x49,  // I
        0x26 => 0x4A,  // J
        0x28 => 0x4B,  // K
        0x25 => 0x4C,  // L
        0x2E => 0x4D,  // M
        0x2D => 0x4E,  // N
        0x1F => 0x4F,  // O
        0x23 => 0x50,  // P
        0x0C => 0x51,  // Q
        0x0F => 0x52,  // R
        0x01 => 0x53,  // S
        0x11 => 0x54,  // T
        0x20 => 0x55,  // U
        0x09 => 0x56,  // V
        0x0D => 0x57,  // W
        0x07 => 0x58,  // X
        0x10 => 0x59,  // Y
        0x06 => 0x5A,  // Z

        // Numbers 0-9
        0x1D => 0x30,  // 0
        0x12 => 0x31,  // 1
        0x13 => 0x32,  // 2
        0x14 => 0x33,  // 3
        0x15 => 0x34,  // 4
        0x17 => 0x35,  // 5
        0x16 => 0x36,  // 6
        0x1A => 0x37,  // 7
        0x1C => 0x38,  // 8
        0x19 => 0x39,  // 9

        // Special keys
        0x24 => VK_RETURN.0 as u16,     // Return
        0x30 => VK_TAB.0 as u16,        // Tab
        0x31 => VK_SPACE.0 as u16,      // Space
        0x33 => VK_BACK.0 as u16,       // Delete/Backspace
        0x35 => VK_ESCAPE.0 as u16,     // Escape
        0x75 => VK_DELETE.0 as u16,     // Forward Delete
        0x73 => VK_HOME.0 as u16,       // Home
        0x77 => VK_END.0 as u16,        // End
        0x74 => VK_PRIOR.0 as u16,      // Page Up
        0x79 => VK_NEXT.0 as u16,       // Page Down

        // Arrow keys
        0x7B => VK_LEFT.0 as u16,       // Left Arrow
        0x7C => VK_RIGHT.0 as u16,      // Right Arrow
        0x7E => VK_UP.0 as u16,         // Up Arrow
        0x7D => VK_DOWN.0 as u16,       // Down Arrow

        // Function keys
        0x7A => VK_F1.0 as u16,
        0x78 => VK_F2.0 as u16,
        0x63 => VK_F3.0 as u16,
        0x76 => VK_F4.0 as u16,
        0x60 => VK_F5.0 as u16,
        0x61 => VK_F6.0 as u16,
        0x62 => VK_F7.0 as u16,
        0x64 => VK_F8.0 as u16,
        0x65 => VK_F9.0 as u16,
        0x6D => VK_F10.0 as u16,
        0x67 => VK_F11.0 as u16,
        0x6F => VK_F12.0 as u16,

        // Punctuation and symbols (German layout specific)
        0x27 => VK_OEM_7.0 as u16,      // ' (Ä on German keyboard)
        0x29 => VK_OEM_3.0 as u16,      // ` (Ö on German keyboard)
        0x21 => VK_OEM_4.0 as u16,      // [ (Ü on German keyboard)
        0x2B => VK_OEM_COMMA.0 as u16,  // ,
        0x2F => VK_OEM_PERIOD.0 as u16, // .
        0x2C => VK_OEM_2.0 as u16,      // /
        0x18 => VK_OEM_PLUS.0 as u16,   // =
        0x1B => VK_OEM_MINUS.0 as u16,  // -

        // Default: pass through
        _ => {
            warn!("Unknown macOS key code: 0x{:02X}, passing through", macos_key);
            macos_key
        }
    }
}

#[cfg(not(windows))]
pub fn simulate_event(_event: &InputEvent) -> Result<()> {
    Err(anyhow!("Input simulation is only supported on Windows"))
}
