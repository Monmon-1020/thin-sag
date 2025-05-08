use crate::adapter::UiAdapter;
use anyhow::{anyhow, Result};
use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

pub struct MacAdapter;

impl MacAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl UiAdapter for MacAdapter {
    fn launch(&self, target: &str) -> Result<()> {
        std::process::Command::new("open")
            .arg("-b")
            .arg(target)
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn click(&self, _sel: Option<&str>, x: Option<i32>, y: Option<i32>) -> Result<()> {
        let (x, y) = (x.unwrap_or(100), y.unwrap_or(100));
        let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| anyhow!("CGEventSource::new failed"))?;
        let mut modifiers = CGEventFlags::empty(); // 修正: modifiers を宣言
        let code = str_to_keycode("key_string", &mut modifiers)?;
        let up_event = CGEvent::new_keyboard_event(src.clone(), code, false)
            .map_err(|_| anyhow::anyhow!("Failed to create keyboard event"))?;
        let pos = CGPoint::new(x as f64, y as f64); // CGPoint を使用
        let down_event = CGEvent::new_mouse_event(
            src.clone(),
            CGEventType::LeftMouseDown,
            pos,
            CGMouseButton::Left,
        )
        .map_err(|_| anyhow!("CGEvent create error"))?;
        let up_event = CGEvent::new_mouse_event(
            src,
            CGEventType::LeftMouseUp,
            pos,
            CGMouseButton::Left, // CGMouseButton::Left を使用
        )
        .map_err(|_| anyhow!("CGEvent create error"))?;

        down_event.post(CGEventTapLocation::HID);
        up_event.post(CGEventTapLocation::HID);

        Ok(())
    }

    fn scroll(&self, dy: i32) -> Result<()> {
        // dy >0 → PageDown, dy<0 → PageUp とする
        let key = if dy < 0 {
            0x74 /* PageUp */
        } else {
            0x79 /* PageDown */
        };
        let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| anyhow!("CGEventSource error"))?;
        let down = CGEvent::new_keyboard_event(src.clone(), key, true)
            .map_err(|_| anyhow!("CGEvent create error"))?;
        let up = CGEvent::new_keyboard_event(src, key, false)
            .map_err(|_| anyhow!("CGEvent create error"))?;
        down.post(CGEventTapLocation::HID);
        up.post(CGEventTapLocation::HID);
        Ok(())
    }

    fn keypress(&self, key: &str) -> Result<()> {
        let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| anyhow!("CGEventSource::new failed"))?;
        let mut modifiers = CGEventFlags::empty();

        for kc in key.split('+') {
            let code = str_to_keycode(kc, &mut modifiers)?;
            let down_event = CGEvent::new_keyboard_event(src.clone(), code, true)
                .map_err(|_| anyhow!("Failed to create keyboard event"))?;
            down_event.post(CGEventTapLocation::HID);
        }

        for kc in key.split('+').rev() {
            let code = str_to_keycode(kc, &mut modifiers)?;
            let up_event = CGEvent::new_keyboard_event(src.clone(), code, false)
                .map_err(|_| anyhow::anyhow!("Failed to create keyboard event"))?;
            up_event.post(CGEventTapLocation::HID);
        }
        Ok(())
    }

    fn type_text(&self, text: &str) -> Result<()> {
        for c in text.chars() {
            let mods = CGEventFlags::empty();
            let code =
                character_to_keycode(c).ok_or_else(|| anyhow!("unsupported char '{}'", c))?;
            CGEvent::new_keyboard_event(
                CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                    .map_err(|_| anyhow!("CGEventSource::new failed"))?,
                code,
                true,
            )
            .map_err(|_| anyhow!("key-down"))?
            .post(CGEventTapLocation::HID);

            CGEvent::new_keyboard_event(
                CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                    .map_err(|_| anyhow!("CGEventSource::new failed"))?,
                code,
                false,
            )
            .map_err(|_| anyhow!("key-up"))?
            .post(CGEventTapLocation::HID);
        }
        Ok(())
    }

    fn wait_ms(&self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

// keycode ユーティリティ関数
fn character_to_keycode(c: char) -> Option<u16> {
    match c {
        'a' => Some(0x00),
        'b' => Some(0x0B),
        'c' => Some(0x08),
        'd' => Some(0x02),
        'e' => Some(0x0E),
        'f' => Some(0x03),
        'g' => Some(0x05),
        'h' => Some(0x04),
        'i' => Some(0x22),
        'j' => Some(0x26),
        'k' => Some(0x28),
        'l' => Some(0x25),
        'm' => Some(0x2E),
        'n' => Some(0x2D),
        'o' => Some(0x1F),
        'p' => Some(0x23),
        'q' => Some(0x0C),
        'r' => Some(0x0F),
        's' => Some(0x01),
        't' => Some(0x11),
        'u' => Some(0x20),
        'v' => Some(0x09),
        'w' => Some(0x0D),
        'x' => Some(0x07),
        'y' => Some(0x10),
        'z' => Some(0x06),
        ' ' => Some(0x31),
        '1' => Some(0x12),
        '2' => Some(0x13),
        '3' => Some(0x14),
        '4' => Some(0x15),
        '5' => Some(0x17),
        '6' => Some(0x16),
        '7' => Some(0x1A),
        '8' => Some(0x1C),
        '9' => Some(0x19),
        '0' => Some(0x1D),
        '-' => Some(0x1B),
        '=' => Some(0x18),
        '[' => Some(0x21),
        ']' => Some(0x1E),
        '\\' => Some(0x2A),
        ';' => Some(0x29),
        '\'' => Some(0x27),
        ',' => Some(0x2B),
        '.' => Some(0x2F),
        '/' => Some(0x2C),
        '`' => Some(0x32),
        _ => None,
    }
}

fn str_to_keycode(s: &str, modifiers: &mut CGEventFlags) -> Result<u16, anyhow::Error> {
    match s.to_lowercase().as_str() {
        "shift" => {
            *modifiers |= CGEventFlags::CGEventFlagShift;
            Ok(0x38)
        }
        "control" => {
            *modifiers |= CGEventFlags::CGEventFlagControl;
            Ok(0x3B)
        }
        "option" => {
            *modifiers |= CGEventFlags::CGEventFlagAlternate;
            Ok(0x3A)
        }
        "command" => {
            *modifiers |= CGEventFlags::CGEventFlagCommand;
            Ok(0x37)
        }
        other => character_to_keycode(other.chars().next().unwrap())
            .ok_or_else(|| anyhow!("Key '{}' not supported", other)),
    }
}
