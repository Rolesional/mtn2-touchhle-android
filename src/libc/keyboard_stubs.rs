/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Native keyboard helpers looked up via dlsym() by some games (e.g. MTN2).

use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::ns_string;
use crate::frameworks::uikit::ui_view::ui_window::{
    UIKeyboardDidHideNotification, UIKeyboardDidShowNotification, UIKeyboardWillHideNotification,
    UIKeyboardWillShowNotification,
};
use crate::mem::{ConstPtr, Mem, MutVoidPtr, Ptr};
use crate::objc::{id, msg, msg_class, nil};
use crate::Environment;

#[derive(Default)]
pub struct State {
    input_text: String,
    callback: Option<GuestFunction>,
    visible: bool,
}

impl State {
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Fresh guest allocation each call — the game may `free()` the pointer.
    fn strdup_input_text(&self, mem: &mut Mem) -> ConstPtr<u8> {
        mem.alloc_and_write_cstr(self.input_text.as_bytes())
            .cast_const()
    }

    fn set_input_text(&mut self, text: String) {
        if self.input_text != text {
            log_dbg!("keyboard input set to {:?}", text);
        }
        self.input_text = text;
    }
}

/// Called when a UITextField's content changes (PC keyboard path).
pub fn set_input_text(env: &mut Environment, text: String) {
    env.libc_state.keyboard_stubs.set_input_text(text);
}

fn post_keyboard_will_show(env: &mut Environment) {
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let name = ns_string::get_static_str(env, UIKeyboardWillShowNotification);
    let _: () = msg![env; center postNotificationName:name object:nil userInfo:nil];
}

fn post_keyboard_did_show(env: &mut Environment) {
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let name = ns_string::get_static_str(env, UIKeyboardDidShowNotification);
    let _: () = msg![env; center postNotificationName:name object:nil userInfo:nil];
}

fn post_keyboard_will_hide(env: &mut Environment) {
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let name = ns_string::get_static_str(env, UIKeyboardWillHideNotification);
    let _: () = msg![env; center postNotificationName:name object:nil userInfo:nil];
}

fn post_keyboard_did_hide(env: &mut Environment) {
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let name = ns_string::get_static_str(env, UIKeyboardDidHideNotification);
    let _: () = msg![env; center postNotificationName:name object:nil userInfo:nil];
}

fn invoke_callback(env: &mut Environment) {
    let callback = env.libc_state.keyboard_stubs.callback;
    let Some(callback) = callback else {
        return;
    };
    log_dbg!("Invoking KB callback {:?}", callback);
    let _: () = callback.call_from_host(env, ());
}

/// Route PC keyboard events while the native KB API keyboard is visible.
pub fn handle_pc_text_input(env: &mut Environment, text_event: crate::window::TextInputEvent) {
    if !env.libc_state.keyboard_stubs.visible {
        return;
    }
    match text_event {
        crate::window::TextInputEvent::Text(text) => {
            let mut combined = env.libc_state.keyboard_stubs.input_text.clone();
            combined.push_str(&text);
            env.libc_state.keyboard_stubs.set_input_text(combined);
            invoke_callback(env);
        }
        crate::window::TextInputEvent::Backspace => {
            env.libc_state
                .keyboard_stubs
                .input_text
                .pop();
            invoke_callback(env);
        }
        crate::window::TextInputEvent::Return => {
            invoke_callback(env);
        }
    }
}

fn KBIsReady(_env: &mut Environment) -> i32 {
    1
}

fn KBIsVisible(env: &mut Environment) -> i32 {
    i32::from(env.libc_state.keyboard_stubs.visible)
}

fn KBShow(env: &mut Environment) -> i32 {
    if env.libc_state.keyboard_stubs.visible {
        return 1;
    }
    log!("KBShow()");
    env.libc_state.keyboard_stubs.visible = true;
    post_keyboard_will_show(env);
    env.on_parent_stack_in_coroutine(|window, _| window.start_text_input());
    post_keyboard_did_show(env);
    invoke_callback(env);
    1
}

fn KBHide(env: &mut Environment) -> i32 {
    if !env.libc_state.keyboard_stubs.visible {
        return 1;
    }
    log!("KBHide()");
    env.libc_state.keyboard_stubs.visible = false;
    post_keyboard_will_hide(env);
    env.on_parent_stack_in_coroutine(|window, _| window.stop_text_input());
    post_keyboard_did_hide(env);
    1
}

/// `const char* KBGetInputText(void)` — must never return NULL.
fn KBGetInputText(env: &mut Environment) -> ConstPtr<u8> {
    let text = env.libc_state.keyboard_stubs.input_text.clone();
    let ptr = env
        .libc_state
        .keyboard_stubs
        .strdup_input_text(&mut env.mem);
    log_dbg!("KBGetInputText() => {:?} ({:?})", ptr, text);
    ptr
}

fn KBGetInputTextLength(env: &mut Environment) -> i32 {
    env.libc_state.keyboard_stubs.input_text.len() as i32
}

fn KBSetInputText(env: &mut Environment, text: ConstPtr<u8>) -> i32 {
    let new_text = if text.is_null() {
        String::new()
    } else {
        env.mem.cstr_at_utf8(text).unwrap_or("").to_string()
    };
    log_dbg!("KBSetInputText({:?})", new_text);
    env.libc_state.keyboard_stubs.set_input_text(new_text);
    invoke_callback(env);
    0
}

fn KBAppendInputText(env: &mut Environment, text: ConstPtr<u8>) -> i32 {
    if text.is_null() {
        return 0;
    }
    let append = env.mem.cstr_at_utf8(text).unwrap_or("");
    let mut combined = env.libc_state.keyboard_stubs.input_text.clone();
    combined.push_str(append);
    log_dbg!("KBAppendInputText({:?}) => {:?}", append, combined);
    env.libc_state.keyboard_stubs.set_input_text(combined);
    invoke_callback(env);
    0
}

fn KBClearInputText(env: &mut Environment) -> i32 {
    log_dbg!("KBClearInputText()");
    env.libc_state.keyboard_stubs.set_input_text(String::new());
    invoke_callback(env);
    0
}

fn KBSetCallback(env: &mut Environment, callback: GuestFunction) -> i32 {
    log_dbg!("KBSetCallback({:?})", callback);
    env.libc_state.keyboard_stubs.callback = Some(callback);
    0
}

/// Generic no-op for any other `_KB*` symbol resolved via dlsym fallback.
fn KBNoOp(_env: &mut Environment) -> i32 {
    1
}

/// Generic pointer return for `_KB*` getters we don't model precisely.
fn KBEmptyString(env: &mut Environment) -> ConstPtr<u8> {
    env.libc_state
        .keyboard_stubs
        .strdup_input_text(&mut env.mem)
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(KBIsReady()),
    export_c_func!(KBIsVisible()),
    export_c_func!(KBShow()),
    export_c_func!(KBHide()),
    export_c_func!(KBGetInputText()),
    export_c_func!(KBGetInputTextLength()),
    export_c_func!(KBSetInputText(_)),
    export_c_func!(KBAppendInputText(_)),
    export_c_func!(KBClearInputText()),
    export_c_func!(KBSetCallback(_)),
    export_c_func!(KBNoOp()),
    export_c_func!(KBEmptyString()),
];

/// If `symbol` is an unimplemented `_KB*` helper, return a safe stub address.
pub fn dlsym_fallback(env: &mut Environment, symbol: &str) -> Option<MutVoidPtr> {
    if !symbol.starts_with("_KB") {
        return None;
    }
    let stub = if symbol.contains("Get") && symbol.contains("Text") && !symbol.contains("Length") {
        "_KBGetInputText"
    } else if symbol.contains("Length") {
        "_KBGetInputTextLength"
    } else if symbol.contains("String") || symbol.contains("Ptr") {
        "_KBEmptyString"
    } else {
        "_KBNoOp"
    };
    log!(
        "dlsym() fallback for unimplemented {symbol} => {stub} stub",
        symbol = symbol,
        stub = stub
    );
    env.dyld
        .create_proc_address(&mut env.mem, &mut env.cpu, stub)
        .ok()
        .map(|f| Ptr::from_bits(f.addr_with_thumb_bit()))
}
