// src/tree.rs

use anyhow::{anyhow, Result};
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::string::CFString;
use core_graphics::display::CGRect;
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};
use serde::Serialize;
use std::mem;
use std::ptr;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut Object;
    fn AXUIElementCopyAttributeValue(
        element: *mut Object,
        attribute: CFTypeRef,
        value: *mut CFTypeRef,
    ) -> i32; // AXError
    fn CFRelease(cf: CFTypeRef);
    fn AXValueGetValue(value: CFTypeRef, valueType: u32, ptr: *mut std::ffi::c_void) -> bool;
}

const kAXValueCGPointType: u32 = 2; // AXValueType for CGPoint
const kAXValueCGSizeType: u32 = 4; // AXValueType for CGSize
const kAXValueCGRectType: u32 = 3; // AXValueType for CGRect

#[derive(Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Serialize)]
pub struct UiNode {
    pub role: String,
    pub label: String,
    pub value: Option<String>,
    pub rect: Option<Rect>,
    pub children: Vec<UiNode>,
}

/// CFTypeRef → Rust String 変換
unsafe fn cf_to_string(cf: CFTypeRef) -> Option<String> {
    if cf.is_null() {
        return None;
    }
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
    // ① まず AXFrame（CGRect）を試す (Window に最適)
    let rect = get_attr(node, "AXFrame")
        .and_then(|cf| {
            // AXValue → CGRect
            let mut frame = mem::MaybeUninit::<CGRect>::uninit();
            let ok =
                unsafe { AXValueGetValue(cf, kAXValueCGRectType, frame.as_mut_ptr() as *mut _) };
            unsafe { CFRelease(cf) };
            if ok {
                let f = unsafe { frame.assume_init() };
                Some(Rect {
                    x: f.origin.x,
                    y: f.origin.y,
                    width: f.size.width,
                    height: f.size.height,
                })
            } else {
                None
            }
        })
        // ② AXFrame 取れなければ Position + Size で補う
        .or_else(|| {
            let pos = get_attr(node, "AXPosition").and_then(|cf| {
                let mut pt = mem::MaybeUninit::<core_graphics::geometry::CGPoint>::uninit();
                let ok =
                    unsafe { AXValueGetValue(cf, kAXValueCGPointType, pt.as_mut_ptr() as *mut _) };
                unsafe { CFRelease(cf) };
                if ok {
                    Some(unsafe { pt.assume_init() })
                } else {
                    None
                }
            });
            let size = get_attr(node, "AXSize").and_then(|cf| {
                let mut sz = mem::MaybeUninit::<core_graphics::geometry::CGSize>::uninit();
                let ok =
                    unsafe { AXValueGetValue(cf, kAXValueCGSizeType, sz.as_mut_ptr() as *mut _) };
                unsafe { CFRelease(cf) };
                if ok {
                    Some(unsafe { sz.assume_init() })
                } else {
                    None
                }
            });
            if let (Some(pt), Some(sz)) = (pos, size) {
                Some(Rect {
                    x: pt.x,
                    y: pt.y,
                    width: sz.width,
                    height: sz.height,
                })
            } else {
                None
            }
        });

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

    UiNode {
        role,
        label,
        value,
        rect,
        children,
    }
}

/// アクティブアプリの AX ツリーを取得
pub fn snapshot_tree() -> Result<UiNode> {
    unsafe {
        // ① 最前面アプリの PID を取得
        let ws_cls = Class::get("NSWorkspace").ok_or_else(|| anyhow!("NSWorkspace not found"))?;
        let ws: *mut Object = msg_send![ws_cls, sharedWorkspace];
        let app: *mut Object = msg_send![ws, frontmostApplication];
        let pid: i32 = msg_send![app, processIdentifier];

        // ② AXUIElementCreateApplication でアプリ要素を作成
        let ax_app = AXUIElementCreateApplication(pid);
        if ax_app.is_null() {
            return Err(anyhow!("AXUIElementCreateApplication failed"));
        }

        // ③ ウィンドウ配列を取得 (AXWindows)
        let windows_cf = get_attr(ax_app, "AXWindows").ok_or_else(|| anyhow!("no AXWindows"))?;
        let arr: *mut Object = mem::transmute(windows_cf);
        let count: usize = msg_send![arr, count];

        // ④ 最初のウィンドウ要素にフォーカス
        if count == 0 {
            CFRelease(windows_cf);
            CFRelease(ax_app as CFTypeRef);
            return Err(anyhow!("no frontmost window"));
        }
        let win: *mut Object = msg_send![arr, objectAtIndex: 0];
        CFRelease(windows_cf);
        CFRelease(ax_app as CFTypeRef);

        // ⑤ build() をウィンドウ要素で呼ぶことで rect を取得可能に
        let root = build(win, 0);
        Ok(root)
    }
}
