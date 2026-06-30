/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `dlfcn.h` (`dlopen()` and friends)

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, MutVoidPtr, Ptr};
use crate::Environment;

const RTLD_DEFAULT: MutVoidPtr = Ptr::from_bits(-2 as _);

fn is_known_library(path: &str) -> bool {
    crate::dyld::DYLIB_LIST
        .iter()
        .any(|dylib| dylib.path == path || dylib.aliases.contains(&path))
}

fn dlopen(env: &mut Environment, path: ConstPtr<u8>, _mode: i32) -> MutVoidPtr {
    if path.is_null() {
        return RTLD_DEFAULT;
    }
    // TODO: dlopen() support for real dynamic libraries.
    assert!(is_known_library(env.mem.cstr_at_utf8(path).unwrap()));
    // For convenience, use the path as the handle.
    // TODO: Find out whether the handle is truly opaque on iPhone OS, and if
    // not, where it points.
    path.cast_mut().cast()
}

fn dlsym_noop(_env: &mut Environment) {}

fn dlsym(env: &mut Environment, handle: MutVoidPtr, symbol: ConstPtr<u8>) -> MutVoidPtr {
    assert!(
        handle == RTLD_DEFAULT || is_known_library(env.mem.cstr_at_utf8(handle.cast()).unwrap())
    );
    // For some reason, the symbols passed to dlsym() don't have the leading _.
    let symbol = format!("_{}", env.mem.cstr_at_utf8(symbol).unwrap());
    // TODO: Symbol lookup should be scoped to the specific library requested,
    // where appropriate!
    //
    // Per the POSIX contract, dlsym() returns NULL when a symbol cannot be
    // found, and well-behaved apps check for this. Some apps (e.g. certain
    // Unity titles) call dlsym() for optional native helpers such as
    // "rating dialog" hooks that touchHLE does not implement; returning NULL
    // lets those apps skip the missing functionality instead of crashing.
    match env
        .dyld
        .create_proc_address(&mut env.mem, &mut env.cpu, &symbol)
    {
        Ok(addr) => Ptr::from_bits(addr.addr_with_thumb_bit()),
        Err(_) => {
            if let Some(addr) = crate::libc::keyboard_stubs::dlsym_fallback(env, &symbol) {
                return addr;
            }
            // MTN2 (and some other titles) call optional native hooks without
            // checking for NULL. A no-op stub avoids a null-page crash.
            if dlsym_should_use_noop_stub(&symbol) {
                log!(
                    "Warning: dlsym() for unimplemented function {symbol}, returning no-op stub",
                );
                return env
                    .dyld
                    .create_proc_address(&mut env.mem, &mut env.cpu, "_dlsym_noop")
                    .ok()
                    .map(|f| Ptr::from_bits(f.addr_with_thumb_bit()))
                    .unwrap_or_else(Ptr::null);
            }
            log!("Warning: dlsym() for unimplemented function {symbol}, returning NULL");
            Ptr::null()
        }
    }
}

fn dlsym_should_use_noop_stub(symbol: &str) -> bool {
    symbol == "_appLaunchedShowRatingDialogIfNeededNative"
        || symbol.contains("ShowRatingDialog")
}

fn dlclose(env: &mut Environment, handle: MutVoidPtr) -> i32 {
    assert!(
        handle == RTLD_DEFAULT || is_known_library(env.mem.cstr_at_utf8(handle.cast()).unwrap())
    );
    0 // success
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(dlopen(_, _)),
    export_c_func!(dlsym(_, _)),
    export_c_func!(dlclose(_)),
    export_c_func!(dlsym_noop()),
];
