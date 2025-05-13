// src/tree.rs

use anyhow::{anyhow, Result};
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::string::CFString;
use core_graphics::display::CGRect;
use globset::Glob;
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};
use serde::Serialize;
use std::mem;
use std::ptr;

const MAX_DEPTH: usize = 8;
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut Object;
    fn AXUIElementCreateSystemWide() -> *mut Object;
    fn AXUIElementCopyAttributeValue(
        element: *mut Object,
        attribute: CFTypeRef,
        value: *mut CFTypeRef,
    ) -> i32;
    static kAXAttributedStringForRangeParameterizedAttribute: CFTypeRef;
    fn AXUIElementCopyParameterizedAttributeValue(
        element: *mut Object,
        parameterizedAttribute: CFTypeRef,
        parameter: CFTypeRef,
        result: *mut CFTypeRef,
    ) -> i32;
    fn CFRelease(cf: CFTypeRef);
    fn AXValueGetValue(value: CFTypeRef, valueType: u32, ptr: *mut std::ffi::c_void) -> bool;
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
}

const kAXValueCGPointType: u32 = 2;
const kAXValueCGSizeType: u32 = 4;
const kAXValueCGRectType: u32 = 3;

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

unsafe fn cf_to_string(cf: CFTypeRef) -> Option<String> {
    if cf.is_null() {
        return None;
    }
    let actual = CFGetTypeID(cf);
    let string_id = CFStringGetTypeID();
    if actual != string_id {
        CFRelease(cf);
        return None;
    }
    let s = CFString::wrap_under_get_rule(cf as _);
    let r = s.to_string();
    CFRelease(cf);
    Some(r)
}

fn get_attr(element: *mut Object, name: &str) -> Option<CFTypeRef> {
    unsafe {
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

unsafe fn list_windows(app_ax: *mut Object) -> Vec<*mut Object> {
    if let Some(cf_arr) = get_attr(app_ax, "AXWindows") {
        let arr: *mut Object = mem::transmute(cf_arr);
        let count: usize = msg_send![arr, count];
        (0..count)
            .map(|i| {
                let w: *mut Object = msg_send![arr, objectAtIndex: i];
                w
            })
            .collect()
    } else {
        vec![]
    }
}

unsafe fn select_window(wins: &[*mut Object], sel: &WindowSelector) -> Option<*mut Object> {
    match sel {
        WindowSelector::Front => wins.first().copied(),
        WindowSelector::Index(i) => wins.get(*i).copied(),
        WindowSelector::Title(glob) => {
            let g = Glob::new(glob).ok()?.compile_matcher();
            wins.iter()
                .find(|w| {
                    get_attr(**w, "AXTitle")
                        .and_then(|cf| cf_to_string(cf))
                        .map(|t| g.is_match(&t))
                        .unwrap_or(false)
                })
                .copied()
        }
        WindowSelector::Doc(path) => wins
            .iter()
            .find(|w| {
                get_attr(**w, "AXDocument")
                    .and_then(|cf| cf_to_string(cf))
                    .map(|p| p == *path)
                    .unwrap_or(false)
            })
            .copied(),
    }
}

#[derive(Debug)]
pub enum WindowSelector {
    Front,
    Index(usize),
    Title(String),
    Doc(String),
}

unsafe fn build(node: *mut Object, depth: usize) -> UiNode {
    // AXRole
    let role = get_attr(node, "AXRole")
        .and_then(|cf| unsafe { cf_to_string(cf) })
        .unwrap_or_default();

    // AXTitle (ラベル)
    let label = get_attr(node, "AXTitle")
        .and_then(|cf| unsafe { cf_to_string(cf) })
        .unwrap_or_default();

    // AXValue またはパラメータ付き属性によるテキスト取得
    let mut value: Option<String> =
        get_attr(node, "AXValue").and_then(|cf| unsafe { cf_to_string(cf) });

    // テキスト系要素の場合、パラメータ付き属性で全文取得
    if value.is_none() {
        match role.as_str() {
            "AXTextArea" | "AXTextField" | "AXStaticText" => {
                // CFRange 全体 (location=0, length=INT_MAX)
                let full_range = {
                    use core_foundation::base::CFRange;
                    let r = CFRange::init(0, isize::MAX);
                    &r as *const _ as CFTypeRef
                };
                // 属性キー
                let key = CFString::new("AXAttributedStringForRangeParameterizedAttribute")
                    .as_CFTypeRef();
                let mut out: CFTypeRef = ptr::null_mut();
                let err =
                    AXUIElementCopyParameterizedAttributeValue(node, key, full_range, &mut out);
                if err == 0 && !out.is_null() {
                    // CFAttributedStringRef → CFString → Rust String
                    if let Some(text) = unsafe { cfattr_to_string(out) } {
                        value = Some(text);
                    }
                }
            }
            _ => {}
        }
    }

    // マスク処理
    let masked_value = value.as_deref().map(crate::mask::mask_text);

    // AXFrame or AXPosition+AXSize から Rect を取得
    let rect = get_attr(node, "AXFrame")
        .and_then(|cf| {
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

    // 子要素を再帰処理
    let mut children = Vec::new();
    if depth < MAX_DEPTH {
        if let Some(cf_children) = get_attr(node, "AXChildren") {
            let arr: *mut Object = mem::transmute(cf_children);
            let count: usize = msg_send![arr, count];
            for i in 0..count {
                let child: *mut Object = msg_send![arr, objectAtIndex: i];
                children.push(build(child, depth + 1));
            }
            unsafe { CFRelease(cf_children) };
        }
    }

    UiNode {
        role,
        label,
        value: masked_value,
        rect,
        children,
    }
}

unsafe fn cfattr_to_string(cf: CFTypeRef) -> Option<String> {
    if cf.is_null() {
        return None;
    }
    let raw: *mut Object = mem::transmute(cf);
    // AttributedString の文字要素を取得
    let cf_str: CFTypeRef = msg_send![raw, string];
    let s = cf_to_string(cf_str)?;
    CFRelease(cf);
    Some(s)
}

#[derive(Serialize)]
pub struct WindowInfo {
    pub index: usize,
    pub title: String,
}

pub fn list_windows_info() -> Vec<WindowInfo> {
    unsafe {
        let ws_cls = Class::get("NSWorkspace").expect("NSWorkspace class not found");
        let ws: *mut Object = msg_send![ws_cls, sharedWorkspace];
        let running_apps: *mut Object = msg_send![ws, runningApplications];
        let app_count: usize = msg_send![running_apps, count];

        let mut infos = Vec::new();
        for app_i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: app_i];
            let pid: i32 = msg_send![app, processIdentifier];
            let ax_app = AXUIElementCreateApplication(pid);
            if ax_app.is_null() {
                continue;
            }
            let wins = list_windows(ax_app);
            for (i, &win) in wins.iter().enumerate() {
                let title = get_attr(win, "AXTitle")
                    .and_then(|cf| unsafe { cf_to_string(cf) })
                    .unwrap_or_else(|| "<no title>".into());
                infos.push(WindowInfo {
                    index: infos.len(),
                    title,
                });
            }
        }
        infos
    }
}

pub fn snapshot_tree(sel: WindowSelector) -> Result<UiNode> {
    unsafe {
        let ws_cls = Class::get("NSWorkspace").ok_or_else(|| anyhow!("NSWorkspace not found"))?;
        let ws: *mut Object = msg_send![ws_cls, sharedWorkspace];
        let running_apps: *mut Object = msg_send![ws, runningApplications];
        let app_count: usize = msg_send![running_apps, count];
        let mut wins = Vec::new();
        for i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
            let pid: i32 = msg_send![app, processIdentifier];
            let ax_app = AXUIElementCreateApplication(pid);
            if !ax_app.is_null() {
                let mut app_wins = list_windows(ax_app);
                wins.append(&mut app_wins);
            }
        }

        eprintln!("DEBUG: system-wide windows count: {}", wins.len());
        for (i, w) in wins.iter().enumerate() {
            let title = get_attr(*w, "AXTitle")
                .and_then(|cf| cf_to_string(cf))
                .unwrap_or_else(|| "<no title>".into());
            eprintln!("  [{}] {}", i, title);
        }
        eprintln!("DEBUG: selector = {:?}", sel);
        let target = select_window(&wins, &sel).ok_or_else(|| {
            eprintln!("DEBUG: no window matched selector {:?}", sel);
            anyhow!("window not found")
        })?;
        Ok(build(target, 0))
    }
}
