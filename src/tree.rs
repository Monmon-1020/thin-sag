// src/tree.rs

use anyhow::{Result, anyhow};
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::string::CFString;
use serde::Serialize;
use std::ptr;
use std::mem;
use objc::runtime::{Object, Class};
use objc::{msg_send, sel, sel_impl};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut Object;
    fn AXUIElementCopyAttributeValue(
        element: *mut Object,
        attribute: CFTypeRef,
        value: *mut CFTypeRef,
    ) -> i32; // AXError
    fn CFRelease(cf: CFTypeRef);
}

#[derive(Serialize)]
pub struct UiNode {
    pub role:  String,
    pub label: String,
    pub value: Option<String>,
    pub children: Vec<UiNode>,
}

/// CFTypeRef → Rust String 変換
unsafe fn cf_to_string(cf: CFTypeRef) -> Option<String> {
    if cf.is_null() { return None }
    // wrap_under_get_rule を使う
    let s = CFString::wrap_under_get_rule(cf as _);
    let r = s.to_string();
    CFRelease(cf);
    Some(r)
}

/// AXUIElement から属性を取得
fn get_attr(element: *mut Object, name: &str) -> Option<CFTypeRef> {
    unsafe {
        // as_CFTypeRef を使う
        let cf_name = CFString::new(name).as_CFTypeRef();
        let mut out: CFTypeRef = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(element, cf_name, &mut out);
        if err == 0 && !out.is_null() {
            Some(out)
        } else {
            None
        }
    }
}

/// 再帰的に AX ツリーを UiNode に変換
unsafe fn build(node: *mut Object, depth: usize) -> UiNode {
    // role
    let role = get_attr(node, "AXRole")
        .and_then(|cf| unsafe { cf_to_string(cf) })
        .unwrap_or_default();
    // label (マスクあり)
    let label = get_attr(node, "AXTitle")
        .and_then(|cf| unsafe { cf_to_string(cf) })
        .as_deref()
        .map(crate::mask::mask_text)
        .unwrap_or_default();

    let value = get_attr(node, "AXValue")
        .and_then(|cf| unsafe { cf_to_string(cf) })
        .as_deref()
        .map(crate::mask::mask_text);

    // children
    let mut children = Vec::new();
    if depth < 3 {
        if let Some(cf_children) = get_attr(node, "AXChildren") {
            let arr: *mut Object = mem::transmute(cf_children);
            let count: usize = msg_send![arr, count];
            for i in 0..count {
                let child: *mut Object = msg_send![arr, objectAtIndex: i];
                children.push(build(child, depth + 1));
            }
            CFRelease(cf_children);
        }
    }

    UiNode { role, label, value, children }
}

/// アクティブアプリの AX ツリーを取得
pub fn snapshot_tree() -> Result<UiNode> {
    unsafe {
        let ws_cls = Class::get("NSWorkspace")
            .ok_or_else(|| anyhow!("NSWorkspace not found"))?;
        let ws: *mut Object = msg_send![ws_cls, sharedWorkspace];
        let app: *mut Object = msg_send![ws, frontmostApplication];
        let pid: i32 = msg_send![app, processIdentifier];

        let ax_app = AXUIElementCreateApplication(pid);
        if ax_app.is_null() {
            return Err(anyhow!("AXUIElementCreateApplication failed"));
        }
        let root = build(ax_app, 0);
        CFRelease(ax_app as CFTypeRef);
        Ok(root)
    }
}
