// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::{model::slide::SlideContent, spawn_show_error};
use leptos::{
    ev::{Custom, message},
    prelude::*,
};
use leptos_use::use_event_listener;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Event, PresentationConnection, PresentationConnectionAvailableEvent,
    PresentationConnectionCloseEvent, PresentationConnectionList,
};

#[component]
pub fn Slide(#[prop(into)] slide: Signal<SlideContent>) -> impl IntoView {
    move || {
        let content = slide.read();
        view! {
            <div class="slide">
            { content.title.as_ref().map(|title| {
                view! {
                    <h1>{title.clone()}</h1>
                }})
            }
            <p>
                { content.lines.iter().map(|line| {
                    let text = line.text.clone();
                    match (line.bold, line.italic) {
                        (false, false) => view! { {text}<br/> }.into_any(),
                        (true, false) => view! { <strong>{text}</strong><br/> }.into_any(),
                        (false, true) => view! { <em>{text}</em><br/> }.into_any(),
                        (true, true) => view! { <strong><em>{text}</em></strong><br/> }.into_any(),
                    }
                } ).collect::<Vec<_>>() }
            </p>
            </div>
        }
    }
}

#[component]
pub fn PresentationReceiver() -> impl IntoView {
    let (current_slide_content, write_current_slide_content) = signal(SlideContent::default());
    let (error, write_error) = signal(None);

    spawn_show_error(
        setup_receiver(write_current_slide_content, write_error),
        write_error,
    );

    view! {
        "Remote"
        <p id="error">{ error }</p>
        <Slide slide=current_slide_content />
    }
}

async fn setup_receiver(
    write_current_slide_content: WriteSignal<SlideContent>,
    write_error: WriteSignal<Option<String>>,
) -> Result<(), String> {
    let presentation = window()
        .navigator()
        .presentation()
        .map_err(|e| format!("{e:?}"))?
        .ok_or("No presentation".to_string())?;
    let receiver = presentation
        .receiver()
        .ok_or("No connection receiver".to_string())?;
    gloo_console::log!(&receiver);
    let connection_list = JsFuture::from(receiver.connection_list().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?
        .unchecked_into::<PresentationConnectionList>();
    gloo_console::log!(&connection_list);

    _ = use_event_listener(
        connection_list.clone(),
        Custom::new("connectionavailable"),
        move |event: PresentationConnectionAvailableEvent| {
            gloo_console::log!(&event);
            setup_connection(event.connection(), write_current_slide_content, write_error);
        },
    );

    let connection = connection_list
        .connections()
        .to_vec()
        .get(0)
        .ok_or("No connection".to_string())?
        .clone()
        .unchecked_into::<PresentationConnection>();
    gloo_console::log!(&connection);

    setup_connection(connection, write_current_slide_content, write_error);

    Ok(())
}

fn setup_connection(
    connection: PresentationConnection,
    write_current_slide_content: WriteSignal<SlideContent>,
    write_error: WriteSignal<Option<String>>,
) {
    _ = use_event_listener(connection.clone(), message, move |event| {
        gloo_console::log!(&event);
        let Some(data) = event.data().as_string() else {
            write_error.set(Some("Data is not a string".to_string()));
            return;
        };
        let Ok(slide) = serde_json::from_str(&data) else {
            write_error.set(Some("Error parsing data".to_string()));
            return;
        };
        write_current_slide_content.set(slide);
    });
    _ = use_event_listener(
        connection.clone(),
        Custom::new("connect"),
        move |event: Event| {
            gloo_console::log!(event);
            write_error.set(Some("connect".to_string()));
        },
    );
    _ = use_event_listener(
        connection.clone(),
        Custom::new("close"),
        move |event: PresentationConnectionCloseEvent| {
            gloo_console::log!(event);
            write_error.set(Some("close".to_string()));
        },
    );
    _ = use_event_listener(
        connection.clone(),
        Custom::new("terminate"),
        move |event: Event| {
            gloo_console::log!(event);
            write_error.set(Some("terminate".to_string()));
        },
    );
}
