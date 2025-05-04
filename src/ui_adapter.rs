use anyhow::Result;
use core_graphics::event::{CGEvent, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::process::Command;

/// Bundle ID でアプリを起動
pub fn launch_app(bundle_id: &str) -> Result<()> {
    // -b で Bundle ID 指定
    Command::new("open")
        .arg("-b")
        .arg(bundle_id)
        .status()
        .map_err(|e| anyhow::anyhow!("open コマンド実行失敗: {}", e))
        .and_then(|st| {
            if st.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("open コマンドが非正常終了: {}", st))
            }
        })
}

/// 文字列を一文字ずつキー入力として送信
pub fn type_text(text: &str) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("CGEventSource の作成に失敗しました"))?;
    for c in text.chars() {
        let keycode = get_keycode(c);
        let down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .map_err(|_| anyhow::anyhow!("CGEvent の作成に失敗しました"))?;
        down.post(CGEventTapLocation::HID);
        let up = CGEvent::new_keyboard_event(source.clone(), keycode, false)
            .map_err(|_| anyhow::anyhow!("CGEvent の作成に失敗しました"))?;
        up.post(CGEventTapLocation::HID);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}

// 文字に対応するキーコードを返す (簡易的な実装)
fn get_keycode(c: char) -> u16 {
    match c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        'i' => 8,
        'j' => 9,
        'k' => 11,
        'l' => 37,
        'm' => 46,
        'n' => 45,
        'o' => 31,
        'p' => 35,
        'q' => 12,
        'r' => 15,
        's' => 1,
        't' => 17,
        'u' => 32,
        'v' => 9,
        'w' => 13,
        'x' => 7,
        'y' => 16,
        'z' => 6,
        ' ' => 49,
        '!' => 33,
        ',' => 43,
        '.' => 47,
        '0' => 29,
        '1' => 18,
        '2' => 19,
        '3' => 20,
        '4' => 21,
        '5' => 23,
        '6' => 22,
        '7' => 26,
        '8' => 28,
        '9' => 25,
        ':' => 41,
        ';' => 39,
        '=' => 30,
        '-' => 27,
        _ => 0,
    }
}