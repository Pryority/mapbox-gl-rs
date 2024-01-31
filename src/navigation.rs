use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;

use crate::*;

macro_rules! impl_handler_for_navigation_control {
    ($(($event:ident, $type:ident),)*) => {
impl NavigationControlHandle {
    pub fn new<F: NavigationControlEventListener + 'static>(control: Weak<NavigationControl>, f: F) -> NavigationControlHandle {
        let f = Rc::new(RefCell::new(f));
        NavigationControlHandle {
            $($event: impl_event_navigation_control!(control, f, $event, $type),)*
        }
    }
}
    }
}

macro_rules! impl_event_navigation_control {
    ($c: ident, $f:ident, $event:ident, JsValue) => {
        Closure::new(enclose!(
            ($c, $f) move |value: JsValue| {
                web_sys::console::debug_2(&JsValue::from(stringify!($event)), &value);
                let Some(control) = $c.upgrade() else {
                    warn!("Failed to get a NavigationControl handle");
                    return;
                };

                match value.try_into() {
                    Ok(e) => {
                        if let Ok(mut f) = $f.try_borrow_mut() {
                            f.deref_mut().$event(control, e);
                        } else {
                            error!("NavigationControl event handler is being called somewhere.");
                        }
                    },
                    Err(e) => {
                        error!("Failed to deserialize Event: {}", e);
                    }
                }
            }
        ))
    };
}

impl_handler_for_navigation_control! {
    (on_click, JsValue),
}

pub trait NavigationControlEventListener {
    fn on_click(&mut self, control: Rc<NavigationControl>, e: event::ClickEvent) {}
    // Add more event methods as needed.
}
pub struct NavigationControlHandle {
    on_click: Closure<dyn Fn(JsValue)>,
    // Add more event closures as needed.
}

pub struct NavigationControl {
    inner: js::NavigationControl,
    handle: RefCell<Option<NavigationControlHandle>>,
}

impl NavigationControl {
    pub fn with_listener<F>(options: NavigationControlOptions, f: F) -> Rc<NavigationControl>
    where
        F: NavigationControlEventListener + 'static,
    {
        let control = Rc::new(NavigationControl {
            inner: js::NavigationControl::new(options.build()),
            handle: RefCell::new(None),
        });

        let handle = NavigationControlHandle::new(Rc::downgrade(&control), f);
        let inner = &control.inner;
        inner.NavigationControl_on("click".into(), &handle.on_click);
        // Add more event listeners as needed.
        control.handle.borrow_mut().replace(handle);

        control
    }
}
