/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSURLConnection`.

use super::{ns_string, NSInteger};
use crate::environment::Environment;
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, msg_super, nil, objc_classes, release, retain, ClassExports,
    HostObject, NSZonePtr, SEL,
};
use std::borrow::Cow;

const NSURLErrorDomain: &str = "NSURLErrorDomain";

/// Our helper type, Foundation just uses ints.
type NSURLErrorCode = NSInteger;
const NSURLErrorNotConnectedToInternet: NSURLErrorCode = -1009;

struct NSURLConnectionHostObject {
    /// `NSURLRequest*`
    request: id,
    /// `id<NSURLConnectionDelegate>`
    delegate: id,
    cancelled: bool,
    started: bool,
}
impl HostObject for NSURLConnectionHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSURLConnection: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSURLConnectionHostObject {
        request: nil,
        delegate: nil,
        cancelled: false,
        started: false,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)sendSynchronousRequest:(id)request // NSURLRequest *
           returningResponse:(MutPtr<id>)response // NSURLResponse **
                       error:(MutPtr<id>)out_error { // NSError **
    log_dbg!(
        "[NSURLConnection sendSynchronousRequest:{:?} ('{}') response:{:?} error:{:?}]",
        request,
        url_string_from_request(env, request),
        response,
        out_error,
    );
    if !response.is_null() {
        env.mem.write(response, nil);
    }
    if !out_error.is_null() {
        if env.options.network_access {
            env.mem.write(out_error, nil);
        } else {
            let domain = ns_string::get_static_str(env, NSURLErrorDomain);
            let error = msg_class![env; NSError alloc];
            let error = msg![env; error initWithDomain:domain code:NSURLErrorNotConnectedToInternet userInfo:nil];
            autorelease(env, error);
            env.mem.write(out_error, error);
        }
    }
    // No NSURLResponse implementation yet — returning data here makes the
    // game take the success path and then dereference a null response.
    nil
}

+ (id)connectionWithRequest:(id)request // NSURLRequest *
                   delegate:(id)delegate {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithRequest:request delegate:delegate];
    autorelease(env, new)
}

- (id)initWithRequest:(id)request // NSURLRequest *
             delegate:(id)delegate {
    msg![env; this initWithRequest:request delegate:delegate startImmediately:true]
}

- (id)initWithRequest:(id)request // NSURLRequest *
             delegate:(id)delegate
     startImmediately:(bool)start_immediately {
    if request != nil && !env.options.network_access {
        log_dbg!(
            "Network access is disabled, [(NSURLConnection *){:?} initWithRequest:{}] -> nil",
            this,
            url_string_from_request(env, request),
        );
        release(env, this);
        return nil;
    }
    let this: id = msg_super![env; this init];
    if request != nil {
        retain(env, request);
    }
    {
        let host_object = env.objc.borrow_mut::<NSURLConnectionHostObject>(this);
        host_object.request = request;
        host_object.delegate = delegate;
        host_object.cancelled = false;
        host_object.started = false;
    }
    log_dbg!(
        "[(NSURLConnection *){:?} initWithRequest:{:?} ('{}') delegate:{:?} startImmediately:{}]",
        this,
        request,
        url_string_from_request(env, request),
        delegate,
        start_immediately,
    );
    if start_immediately && request != nil {
        start_connection_loading(env, this);
    }
    this
}

- (())start {
    let started = env.objc.borrow::<NSURLConnectionHostObject>(this).started;
    if !started {
        start_connection_loading(env, this);
    }
}

- (())cancel {
    env.objc.borrow_mut::<NSURLConnectionHostObject>(this).cancelled = true;
}

@end

};

fn start_connection_loading(env: &mut Environment, connection: id) {
    let (delegate, cancelled) = {
        let host_object = env.objc.borrow_mut::<NSURLConnectionHostObject>(connection);
        if host_object.started {
            return;
        }
        host_object.started = true;
        (host_object.delegate, host_object.cancelled)
    };
    if cancelled || delegate == nil {
        return;
    }

    let finish_sel: SEL = env
        .objc
        .register_host_selector("connectionDidFinishLoading:".to_string(), &mut env.mem);
    if msg![env; delegate respondsToSelector:finish_sel] {
        () = msg![env; delegate performSelector:finish_sel withObject:connection afterDelay:0.0];
    }
}

fn url_string_from_request(env: &mut Environment, request: id) -> Cow<'static, str> {
    if request == nil {
        Cow::from("(null)")
    } else {
        let url = msg![env; request URL];
        let ns_string = msg![env; url absoluteString];
        ns_string::to_rust_string(env, ns_string)
    }
}
