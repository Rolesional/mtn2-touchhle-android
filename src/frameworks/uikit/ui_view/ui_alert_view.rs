/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIAlertView`.

use crate::frameworks::core_graphics::cg_affine_transform::CGAffineTransform;
use crate::frameworks::core_graphics::{CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_string;
use crate::frameworks::foundation::{NSInteger, NSUInteger};
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, ClassExports, NSZonePtr,
};

pub type UIAlertViewStyle = NSInteger;
pub const UIAlertViewStyleDefault: UIAlertViewStyle = 0;
pub const UIAlertViewStyleSecureTextInput: UIAlertViewStyle = 1;
pub const UIAlertViewStylePlainTextInput: UIAlertViewStyle = 2;
pub const UIAlertViewStyleLoginAndPasswordInput: UIAlertViewStyle = 3;

pub struct UIAlertViewHostObject {
    superclass: super::UIViewHostObject,
    title: id,
    message: id,
    delegate: id,
    button_titles: id,
    cancel_button_index: NSInteger,
    visible: bool,
    alert_view_style: UIAlertViewStyle,
    tag: NSInteger,
    text_field: id,
}
impl_HostObject_with_superclass!(UIAlertViewHostObject);

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIAlertView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIAlertViewHostObject {
        superclass: Default::default(),
        title: nil,
        message: nil,
        delegate: nil,
        button_titles: nil,
        cancel_button_index: -1,
        visible: false,
        alert_view_style: UIAlertViewStyleDefault,
        tag: 0,
        text_field: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithTitle:(id)title
                      message:(id)message
                     delegate:(id)delegate
            cancelButtonTitle:(id)cancel_title
            otherButtonTitles:(id)other_titles {

    let this: id = msg_super![env; this init];

    let buttons: id = msg_class![env; NSMutableArray new];
    retain(env, title);
    retain(env, message);
    {
        let host = env.objc.borrow_mut::<UIAlertViewHostObject>(this);
        host.title = title;
        host.message = message;
        host.delegate = delegate;
        host.button_titles = buttons;
    }
    if cancel_title != nil {
        let idx: NSUInteger = msg![env; buttons count];
        let _: () = msg![env; buttons addObject:cancel_title];
        env.objc.borrow_mut::<UIAlertViewHostObject>(this).cancel_button_index = idx as NSInteger;
    }
    if other_titles != nil {
        let _: () = msg![env; buttons addObject:other_titles];
    }

    let title_str = if title != nil {
        ns_string::to_rust_string(env, title).into_owned()
    } else {
        "(nil)".into()
    };
    let msg_str = if message != nil {
        ns_string::to_rust_string(env, message).into_owned()
    } else {
        "(nil)".into()
    };
    log!("UIAlertView init title={:?} message={:?}", title_str, msg_str);
    this
}

- (())dealloc {
    let (title, message, buttons, text_field) = {
        let host = env.objc.borrow::<UIAlertViewHostObject>(this);
        (host.title, host.message, host.button_titles, host.text_field)
    };
    release(env, title);
    release(env, message);
    release(env, buttons);
    release(env, text_field);
    msg_super![env; this dealloc]
}

- (id)title {
    env.objc.borrow::<UIAlertViewHostObject>(this).title
}
- (id)message {
    env.objc.borrow::<UIAlertViewHostObject>(this).message
}
- (id)delegate {
    env.objc.borrow::<UIAlertViewHostObject>(this).delegate
}

- (())setTitle:(id)title {
    let old = env.objc.borrow::<UIAlertViewHostObject>(this).title;
    release(env, old);
    retain(env, title);
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).title = title;
}
- (())setMessage:(id)message {
    let old = env.objc.borrow::<UIAlertViewHostObject>(this).message;
    release(env, old);
    retain(env, message);
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).message = message;
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).delegate = delegate;
}

- (NSInteger)tag {
    env.objc.borrow::<UIAlertViewHostObject>(this).tag
}
- (())setTag:(NSInteger)tag {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).tag = tag;
}
- (UIAlertViewStyle)alertViewStyle {
    env.objc.borrow::<UIAlertViewHostObject>(this).alert_view_style
}
- (())setAlertViewStyle:(UIAlertViewStyle)style {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).alert_view_style = style;
}
- (bool)isVisible {
    env.objc.borrow::<UIAlertViewHostObject>(this).visible
}

- (NSInteger)addButtonWithTitle:(id)title {
    let buttons = env.objc.borrow::<UIAlertViewHostObject>(this).button_titles;
    let idx: NSUInteger = msg![env; buttons count];
    let _: () = msg![env; buttons addObject:title];
    idx as NSInteger
}
- (NSUInteger)numberOfButtons {
    let buttons = env.objc.borrow::<UIAlertViewHostObject>(this).button_titles;
    msg![env; buttons count]
}
- (id)buttonTitleAtIndex:(NSInteger)index {
    let buttons = env.objc.borrow::<UIAlertViewHostObject>(this).button_titles;
    let count: NSUInteger = msg![env; buttons count];
    if index < 0 || index as NSUInteger >= count {
        return nil;
    }
    msg![env; buttons objectAtIndex:(index as NSUInteger)]
}
- (NSInteger)cancelButtonIndex {
    env.objc.borrow::<UIAlertViewHostObject>(this).cancel_button_index
}
- (())setCancelButtonIndex:(NSInteger)index {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).cancel_button_index = index;
}
- (NSInteger)firstOtherButtonIndex {
    let (buttons, cancel) = {
        let host = env.objc.borrow::<UIAlertViewHostObject>(this);
        (host.button_titles, host.cancel_button_index)
    };
    let count: NSUInteger = msg![env; buttons count];
    for i in 0..count {
        if i as NSInteger != cancel {
            return i as NSInteger;
        }
    }
    -1
}

- (id)textFieldAtIndex:(NSInteger)index {
    if index != 0 {
        return nil;
    }
    let style = env.objc.borrow::<UIAlertViewHostObject>(this).alert_view_style;
    if style == UIAlertViewStyleDefault {
        return nil;
    }
    let mut text_field = env.objc.borrow::<UIAlertViewHostObject>(this).text_field;
    if text_field == nil {
        text_field = msg_class![env; UITextField new];
        retain(env, text_field);
        env.objc.borrow_mut::<UIAlertViewHostObject>(this).text_field = text_field;
    }
    text_field
}

- (())addSubview:(id)_view {
    log_dbg!("UIAlertView addSubview: ignored");
}
- (())removeFromSuperview {
    log_dbg!("UIAlertView removeFromSuperview: ignored");
}
- (())setHidden:(bool)_hidden {
    log_dbg!("UIAlertView setHidden: ignored");
}
- (CGRect)frame {
    CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize {
            width: 0.0,
            height: 0.0,
        },
    }
}
- (())setFrame:(CGRect)_frame {
    log_dbg!("UIAlertView setFrame: ignored");
}
- (())setTransform:(CGAffineTransform)_transform {
    log_dbg!("UIAlertView setTransform: ignored");
}
- (id)viewWithTag:(NSInteger)tag {
    let own_tag: NSInteger = msg![env; this tag];
    if own_tag == tag {
        return this;
    }
    nil
}
- (CGSize)sizeThatFits:(CGSize)size {
    size
}
- (())sizeToFit {
    let frame: CGRect = msg![env; this frame];
    let current_size = frame.size;
    let new_size: CGSize = msg![env; this sizeThatFits:current_size];
    let new_frame = CGRect {
        origin: frame.origin,
        size: new_size,
    };
    let _: () = msg![env; this setFrame:new_frame];
}

- (())show {
    log!("UIAlertView show");
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).visible = true;

    let (title, message, cancel_index, delegate) = {
        let h = env.objc.borrow::<UIAlertViewHostObject>(this);
        (h.title, h.message, h.cancel_button_index, h.delegate)
    };

    if delegate != nil {
        if let Some(sel) = env.objc.lookup_selector("alertViewWillPresent:") {
            let responds: bool = msg![env; delegate respondsToSelector:sel];
            if responds {
                let _: () = msg![env; delegate alertViewWillPresent:this];
            }
        }
    }

    let raw_title: String = if title != nil {
        ns_string::to_rust_string(env, title).into_owned()
    } else {
        String::new()
    };
    let raw_message: String = if message != nil {
        ns_string::to_rust_string(env, message).into_owned()
    } else {
        String::new()
    };

    // Real iOS show is non-blocking. Empty placeholder alerts should dismiss
    // immediately so the guest run loop keeps going.
    if raw_title.trim().is_empty() && raw_message.trim().is_empty() {
        log!("UIAlertView show: empty title and message; dismissing asynchronously");
        let dismiss_index = if cancel_index >= 0 { cancel_index } else { 0 };
        let _: () = msg![env; this dismissWithClickedButtonIndex:dismiss_index animated:false];
        return;
    }

    // MTN2 and similar titles show a transient "INITIALIZING KEYBOARD..."
    // placeholder alert while setting up their custom keyboard UI. Dismiss it
    // immediately so the game can continue (real iOS does not block here).
    if raw_title.to_ascii_uppercase().contains("INITIALIZING")
        || raw_message.to_ascii_uppercase().contains("INITIALIZING")
    {
        log!(
            "UIAlertView show: transient init alert {:?}; dismissing asynchronously",
            raw_title
        );
        let dismiss_index = if cancel_index >= 0 { cancel_index } else { 0 };
        let _: () = msg![env; this dismissWithClickedButtonIndex:dismiss_index animated:false];
    }
}

- (())dismissWithClickedButtonIndex:(NSInteger)button_index animated:(bool)_animated {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).visible = false;
    let delegate = env.objc.borrow::<UIAlertViewHostObject>(this).delegate;

    if delegate != nil {
        let isa: u32 = env.mem.read(delegate.cast());
        if isa != 0 {
            if let Some(sel) = env
                .objc
                .lookup_selector("alertView:clickedButtonAtIndex:")
            {
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let _: () = msg![env; delegate alertView:this clickedButtonAtIndex:button_index];
                }
            }
            if let Some(sel) = env
                .objc
                .lookup_selector("alertView:willDismissWithButtonIndex:")
            {
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let _: () = msg![env; delegate alertView:this willDismissWithButtonIndex:button_index];
                }
            }
            if let Some(sel) = env
                .objc
                .lookup_selector("alertView:didDismissWithButtonIndex:")
            {
                let responds: bool = msg![env; delegate respondsToSelector:sel];
                if responds {
                    let _: () = msg![env; delegate alertView:this didDismissWithButtonIndex:button_index];
                }
            }
        }
    }
}

@end

};
