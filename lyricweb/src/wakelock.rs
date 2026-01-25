// Copyright 2026 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use leptos::{ev::visibilitychange, prelude::*, task::spawn_local};
use leptos_use::use_event_listener;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{VisibilityState, WakeLockSentinel, WakeLockType};

#[derive(Debug)]
pub struct WakeLockGuard {
    sentinel: RefCell<Option<WakeLockSentinel>>,
}

impl WakeLockGuard {
    /// Requests the screen wakelock.
    pub fn new() -> Rc<Self> {
        let guard = Rc::new(Self {
            sentinel: RefCell::new(None),
        });

        spawn_local(guard.clone().request_and_save());

        let guard_clone = guard.clone();

        _ = use_event_listener(document(), visibilitychange, move |_| {
            if document().visibility_state() == VisibilityState::Visible {
                spawn_local(guard_clone.clone().request_and_save());
            }
        });

        guard
    }

    async fn request_and_save(self: Rc<WakeLockGuard>) {
        match request().await {
            Ok(sentinel) => {
                self.sentinel.borrow_mut().replace(sentinel);
            }
            Err(e) => gloo_console::log!(e),
        }
    }
}

async fn request() -> Result<WakeLockSentinel, JsValue> {
    Ok(JsFuture::from(
        window()
            .navigator()
            .wake_lock()
            .request(WakeLockType::Screen),
    )
    .await?
    .unchecked_into::<WakeLockSentinel>())
}

impl Drop for WakeLockGuard {
    fn drop(&mut self) {
        if let Some(sentinel) = self.sentinel.take() {
            if !sentinel.released() {
                spawn_local(release_sentinel(sentinel));
            }
        }
    }
}

async fn release_sentinel(sentinel: WakeLockSentinel) {
    _ = JsFuture::from(sentinel.release()).await;
}
