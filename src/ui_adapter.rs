use anyhow::Result;
use core_foundation::string::CFString;
use objc::{class, msg_send, sel, sel_impl};
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType};
use core_graphics::event_source::CGEventSourceStateID;
use core_graphics::event_source::CGEventSource;
use core_foundation::base::TCFType;
use objc::runtime::Class;
use anyhow::anyhow;
/// Bundle ID でアプリを起動

pub fn launch_app(bundle_id: &str) -> Result<()> {
    unsafe {
        let ns_workspace = Class::get("NSWorkspace")
            .ok_or_else(|| anyhow!("NSWorkspace クラスが見つかりません"))?;
        let workspace: *mut objc::runtime::Object = msg_send![ns_workspace, sharedWorkspace];
        let () = msg_send![workspace,
            launchAppWithBundleIdentifier: CFString::new(bundle_id).as_CFType()
            options: 0
            configuration: std::ptr::null_mut::<objc::runtime::Object>()
        ];
    }
    Ok(())
}

/// 文字列を一文字ずつキー入力として送信
pub fn type_text(text: &str) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("CGEventSource の作成に失敗しました"))?;
    for c in text.chars() {
        // ここでは仮に 'a' のキーコード (0) を使用 - 実際には各文字に対応するキーコードを取得する必要があります
        let keycode = get_keycode(c);
        let down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .map_err(|_| anyhow::anyhow!("CGEvent の作成に失敗しました"))?;
        down.post(CGEventTapLocation::HID);
        let up = CGEvent::new_keyboard_event(source.clone(), keycode, false)
            .map_err(|_| anyhow::anyhow!("CGEvent の作成に失敗しました"))?;
        up.post(CGEventTapLocation::HID);
        std::thread::sleep(std::time::Duration::from_millis(50)); // キー連打にならないように少し待つ
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
        '!' => 33, // Shift + 1
        ',' => 43,
        '.' => 47,
        _ => 0, // デフォルトは 'a'
    }
}