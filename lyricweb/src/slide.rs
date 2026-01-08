// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::{model::slide::SlideContent, spawn_show_error};
use leptos::{
    ev::{Custom, message},
    prelude::*,
};
use leptos_meta::Style;
use leptos_use::use_event_listener;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Event, KeyboardEvent, PresentationConnection, PresentationConnectionCloseEvent,
    PresentationConnectionList, PresentationConnectionState,
};

#[component]
pub fn Slide(#[prop(into)] slide: Signal<SlideContent>) -> impl IntoView {
    move || {
        let content = slide.read();
        let theme = content.theme.clone();
        view! {
            <Style>
            ".slide {"
                "background-color: " {theme.background_colour} ";"
                "font-family: " {theme.font_family} ";"
            "}"
            ".slide h1 {"
                "font-size:" {theme.heading_size as f32 / 10.0}"cqi;"
                "color: " {theme.heading_colour} ";"
            "}"
            ".slide h2 {"
                "font-size:" {theme.body_size as f32 / 10.0}"cqi;"
                "color: " {theme.body_colour.clone()} ";"
            "}"
            ".slide p {"
                "font-size:" {theme.body_size as f32 / 10.0}"cqi;"
                "color: " {theme.body_colour} ";"
            "}"
            ".slide p.credit {"
                "font-size:" {theme.body_size as f32 / 20.0} "cqi;"
            "}"
            </Style>
            <div class="slide">
            { content.title.as_ref().map(|title| {
                view! {
                    <h1>{title.clone()}</h1>
                }
            })}
            { content.body.as_ref().map(|body| {
                view! {
                    <div inner_html=body.clone() />
                }
            })}
            { content.credit.as_ref().map(|credit| {
                view! {
                    <p class="credit">{credit.clone()}</p>
                }
            })}
            </div>
        }
    }
}

#[component]
pub fn PresentationReceiver() -> impl IntoView {
    let (current_slide_content, write_current_slide_content) = signal(SlideContent::default());
    let (error, write_error) = signal(None);

    let connection = StoredValue::new_local(None);
    spawn_show_error(
        setup_receiver(write_current_slide_content, write_error, connection),
        write_error,
    );

    view! {
        <div id="presentation" tabindex="0" on:keydown=move |event| presentation_receiver_keydown(event, connection)>
            <p id="error">{ error }</p>
            <Slide slide=current_slide_content />
        </div>
    }
}

async fn setup_receiver(
    write_current_slide_content: WriteSignal<SlideContent>,
    write_error: WriteSignal<Option<String>>,
    stored_connection: StoredValue<Option<PresentationConnection>, LocalStorage>,
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

    let connection = connection_list
        .connections()
        .to_vec()
        .get(0)
        .ok_or("No connection".to_string())?
        .clone()
        .unchecked_into::<PresentationConnection>();
    gloo_console::log!(&connection);

    setup_connection(&connection, write_current_slide_content, write_error);

    stored_connection.set_value(Some(connection));

    Ok(())
}

fn setup_connection(
    connection: &PresentationConnection,
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
    if connection.state() == PresentationConnectionState::Connected {
        connection.send_with_str("").unwrap();
    }
}

fn presentation_receiver_keydown(
    event: KeyboardEvent,
    connection: StoredValue<Option<PresentationConnection>, LocalStorage>,
) {
    if let Some(connection) = connection.read_value().as_ref()
        && connection.state() == PresentationConnectionState::Connected
    {
        match event.key().as_str() {
            "ArrowLeft" => {
                event.prevent_default();
                connection.send_with_str("prev").unwrap();
            }
            "ArrowRight" => {
                event.prevent_default();
                connection.send_with_str("next").unwrap();
            }
            _ => {}
        }
    }
}
