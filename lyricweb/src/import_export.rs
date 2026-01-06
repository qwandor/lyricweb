// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::{
    files::{FileType, pick_open_file, pick_save_file, write_and_close},
    model::State,
};
use abc_parser::abc::tune_book;
use gloo_file::{File, futures::read_as_text};
use gloo_net::http::Request;
use gloo_utils::format::JsValueSerdeExt;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use lyricutils::tunebook_to_open_lyrics;
use wasm_bindgen::JsValue;
use web_sys::{
    FileSystemWritableFileStream, OpenFilePickerOptions, SaveFilePickerOptions, SubmitEvent,
};

/// Exports the state to a file.
pub async fn export(event: SubmitEvent, state: Signal<State>) -> Result<(), String> {
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
        return Ok(());
    };

    export_to_file(state, file).await?;

    Ok(())
}

async fn export_to_file(
    state: Signal<State>,
    file: FileSystemWritableFileStream,
) -> Result<(), String> {
    let state = state.read_untracked();
    write_and_close(
        &file,
        &serde_json::to_string::<State>(&state).map_err(|e| e.to_string())?,
    )
    .await
    .map_err(|e| format!("{e:?}"))
}

/// Imports a single song or the entire state from a URL, and then redirect to the main page.
pub async fn import_url(
    event: SubmitEvent,
    url: String,
    write_state: WriteSignal<State>,
    navigate: impl Fn(&str, NavigateOptions) + Clone,
) -> Result<(), String> {
    event.prevent_default();

    let response = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(format!("Error: {}", response.status_text()));
    }

    let body = response.text().await.map_err(|e| e.to_string())?;
    import_str(Format::from_filename(&url), &body, write_state)?;

    navigate(".", Default::default());
    Ok(())
}

/// Imports a single song or the entire state from a file.
pub async fn import(
    event: SubmitEvent,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
) -> Result<(), String> {
    event.prevent_default();

    let options = OpenFilePickerOptions::new();
    options.set_id("import");
    options.set_types(
        &JsValue::from_serde(&[
            FileType {
                description: Some("ABC, JSON or XML file".to_string()),
                accept: [
                    ("text/vnd.abc".to_string(), vec![".abc".to_string()]),
                    ("application/json".to_string(), vec![".json".to_string()]),
                    ("text/xml".to_string(), vec![".xml".to_string()]),
                ]
                .into_iter()
                .collect(),
            },
            FileType {
                description: Some("ABC file".to_string()),
                accept: [("text/vnd.abc".to_string(), vec![".abc".to_string()])]
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
        return Ok(());
    };

    import_file(file, write_state, write_output).await?;

    Ok(())
}

async fn import_file(
    file: File,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
) -> Result<(), String> {
    write_output.set(Some(format!(
        "{}: {} bytes, {}",
        file.name(),
        file.size(),
        file.raw_mime_type()
    )));
    let text = read_as_text(&file).await.map_err(|e| e.to_string())?;
    import_str(Format::from_filename(&file.name()), &text, write_state)
}

fn import_str(format: Format, text: &str, write_state: WriteSignal<State>) -> Result<(), String> {
    match format {
        Format::Json => {
            let imported_state = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            write_state.update(|state| state.merge(&imported_state));
        }
        Format::Xml => {
            let song = quick_xml::de::from_str(&text).map_err(|e| e.to_string())?;
            write_state.update(|state| {
                state.add_song(song);
            });
        }
        Format::Abc => {
            let tunebook = tune_book(text).map_err(|e| e.to_string())?;
            let song = tunebook_to_open_lyrics(&tunebook);
            write_state.update(|state| {
                state.add_song(song);
            });
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Format {
    Json,
    Xml,
    Abc,
}

impl Format {
    fn from_filename(filename: &str) -> Self {
        if filename.ends_with(".json") {
            Self::Json
        } else if filename.ends_with(".xml") {
            Self::Xml
        } else {
            Self::Abc
        }
    }
}
