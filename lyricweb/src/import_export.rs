// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::{
    files::{FileType, pick_open_file, pick_save_file, write_and_close},
    model::State,
};
use gloo_file::{File, futures::read_as_text};
use gloo_utils::format::JsValueSerdeExt;
use leptos::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{OpenFilePickerOptions, SaveFilePickerOptions, SubmitEvent};

/// Exports the state to a file.
pub async fn export(
    event: SubmitEvent,
    state: Signal<State>,
    write_error: WriteSignal<Option<String>>,
) {
    event.prevent_default();

    let options = SaveFilePickerOptions::new();
    options.set_id("export");
    options.set_suggested_name(Some("lyricweb.json"));
    options.set_types(
        &JsValue::from_serde(&[FileType {
            description: Some("JSON file".to_string()),
            accept: [("application/json".to_string(), vec![".json".to_string()])]
                .into_iter()
                .collect(),
        }])
        .unwrap(),
    );

    let Ok(file) = pick_save_file(&options).await else {
        return;
    };

    let state = state.read_untracked();
    if let Err(e) = write_and_close(&file, &serde_json::to_string::<State>(&state).unwrap()).await {
        write_error.set(Some(format!("{e:?}")));
    } else {
        write_error.set(None);
    }
}

/// Imports a single song or the entire state from a file.
pub async fn import(
    event: SubmitEvent,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
    write_error: WriteSignal<Option<String>>,
) {
    event.prevent_default();

    let options = OpenFilePickerOptions::new();
    options.set_id("import");
    options.set_types(
        &JsValue::from_serde(&[
            FileType {
                description: Some("JSON or XML file".to_string()),
                accept: [
                    ("application/json".to_string(), vec![".json".to_string()]),
                    ("text/xml".to_string(), vec![".xml".to_string()]),
                ]
                .into_iter()
                .collect(),
            },
            FileType {
                description: Some("JSON file".to_string()),
                accept: [("application/json".to_string(), vec![".json".to_string()])]
                    .into_iter()
                    .collect(),
            },
            FileType {
                description: Some("XML file".to_string()),
                accept: [("text/xml".to_string(), vec![".xml".to_string()])]
                    .into_iter()
                    .collect(),
            },
        ])
        .unwrap(),
    );

    let Ok(file) = pick_open_file(&options).await else {
        return;
    };

    import_file(file, write_state, write_output, write_error).await;
}

async fn import_file(
    file: File,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
    write_error: WriteSignal<Option<String>>,
) {
    write_output.set(Some(format!(
        "{}: {} bytes, {}",
        file.name(),
        file.size(),
        file.raw_mime_type()
    )));
    let text = read_as_text(&file).await.unwrap();
    if file.name().ends_with(".json") {
        match serde_json::from_str(&text) {
            Ok(imported_state) => write_state.update(|state| state.merge(&imported_state)),
            Err(e) => write_error.set(Some(e.to_string())),
        }
    } else {
        match quick_xml::de::from_str(&text) {
            Ok(song) => {
                write_error.set(None);
                write_state.update(|state| {
                    state.add_song(song);
                });
            }
            Err(e) => write_error.set(Some(e.to_string())),
        }
    }
}
