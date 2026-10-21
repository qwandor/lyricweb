// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use leptos::{ev::Custom, prelude::*, task::spawn_local};
use leptos_use::use_event_listener;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, ScreenDetailed, ScreenDetails};

/// Returns a signal which lists the currently connected screens, if they are available.
pub fn use_screens() -> ReadSignal<Vec<ScreenDetailed>, LocalStorage> {
    let (screens, write_screens) = signal_local(Vec::new());
    spawn_local(async move {
        let Ok(details) = get_screen_details().await else {
            return;
        };

        // Set an initial value.
        write_screens.set(get_screens_detailed(&details));

        // Listen for updates.
        _ = use_event_listener(
            details.clone(),
            Custom::new("screenschange"),
            move |_: Event| {
                write_screens.set(get_screens_detailed(&details));
            },
        );
    });
    screens
}

async fn get_screen_details() -> Result<ScreenDetails, JsValue> {
    Ok(JsFuture::from(window().get_screen_details()?)
        .await?
        .unchecked_into::<ScreenDetails>())
}

fn get_screens_detailed(details: &ScreenDetails) -> Vec<ScreenDetailed> {
    details
        .screens()
        .iter()
        .map(|screen| screen.unchecked_into::<ScreenDetailed>())
        .collect()
}
