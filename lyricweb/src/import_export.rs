// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::{
    files::{make_accept_type, pick_open_file, pick_save_file, write_and_close},
    model::State,
};
use abc_parser::abc::tune_book;
use gloo_file::{File, futures::read_as_text};
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use lyricutils::{music_xml_to_open_lyrics, tunebook_to_open_lyrics};
use musicxml::read_score_data_partwise;
use web_sys::{
    FileSystemWritableFileStream, OpenFilePickerOptions, SaveFilePickerOptions, SubmitEvent,
};

/// Exports the state to a file.
pub async fn export(event: SubmitEvent, state: Signal<State>) -> Result<(), String> {
    event.prevent_default();

    let options = SaveFilePickerOptions::new();
    options.set_id("export");
    options.set_suggested_name(Some("lyricweb.json"));
    options.set_types(&[make_accept_type(
        "JSON file",
        &[("application/json", &[".json"])],
    )]);

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
    options.set_types(&[
        make_accept_type(
            "ABC, JSON or XML file",
            &[
                ("text/vnd.abc", &[".abc"]),
                ("application/json", &[".json"]),
                ("text/xml", &[".xml"]),
            ],
        ),
        make_accept_type("ABC file", &[("text/vnd.abc", &[".abc"])]),
        make_accept_type("JSON file", &[("application/json", &[".json"])]),
        make_accept_type("XML file", &[("text/xml", &[".xml"])]),
    ]);

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
        Format::MusicXml => {
            let score = read_score_data_partwise(text.into())?;
            let song = music_xml_to_open_lyrics(&score);
            write_state.update(|state| {
                state.add_song(song);
            });
        }
        Format::Abc => {
            let tunebook = tune_book(text.trim()).map_err(|e| e.to_string())?;
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
    MusicXml,
    Xml,
    Abc,
}

impl Format {
    fn from_filename(filename: &str) -> Self {
        if filename.ends_with(".json") {
            Self::Json
        } else if filename.ends_with(".mxl") || filename.ends_with(".musicxml") {
            Self::MusicXml
        } else if filename.ends_with(".xml") {
            Self::Xml
        } else {
            Self::Abc
        }
    }
}
