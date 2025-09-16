// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod model;

use crate::model::{PlaylistEntry, Slide, State, title_for_song};
use gloo_file::{File, FileList, futures::read_as_text};
use gloo_utils::document;
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};
use quick_xml::de::from_str;
use std::{fmt::Write, sync::Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, Event, EventTarget, HtmlInputElement, HtmlSelectElement};

static STATE: Mutex<State> = Mutex::new(State::new());

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    add_async_listener_by_id("file", "change", file_changed);
    add_listener_by_id("song_list_form", "submit", add_song_to_playlist);
    add_listener_by_id("text_form", "submit", add_text_to_playlist);
    add_listener_by_id("playlist", "change", playlist_entry_selected);
}

fn get_element_by_id(id: &str) -> Element {
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Failed to find element {id}"))
}

fn add_async_listener_by_id<F: Future<Output = ()> + 'static>(
    id: &str,
    event_type: &str,
    callback: fn(Event) -> F,
) {
    add_listener_by_id(id, event_type, move |event| spawn_local(callback(event)));
}

fn add_listener_by_id(id: &str, event_type: &str, callback: impl Fn(Event) + 'static) {
    add_listener_and_leak(&get_element_by_id(id), event_type, callback);
}

fn add_listener_and_leak(
    target: &EventTarget,
    event_type: &str,
    callback: impl Fn(Event) + 'static,
) {
    target
        .add_event_listener_with_callback(
            event_type,
            Closure::<dyn Fn(Event)>::new(callback)
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap();
}

fn add_song_to_playlist(event: Event) {
    event.prevent_default();

    let selected = document()
        .get_element_by_id("song_list")
        .expect("Couldn't find song_list")
        .unchecked_into::<HtmlSelectElement>()
        .selected_index();
    show_output(&format!("selected: {selected}"));
    if selected >= 0 {
        STATE.lock().unwrap().playlist.push(PlaylistEntry::Song {
            song_index: selected.try_into().unwrap(),
        });
        update_playlist();
    }
}

fn add_text_to_playlist(event: Event) {
    event.prevent_default();

    let text = document()
        .get_element_by_id("text_entry")
        .expect("Couldn't find text_entry")
        .unchecked_into::<HtmlInputElement>()
        .value();
    STATE
        .lock()
        .unwrap()
        .playlist
        .push(PlaylistEntry::Text(text));
    update_playlist();
}

async fn file_changed(_event: Event) {
    let files = FileList::from(
        get_element_by_id("file")
            .unchecked_into::<HtmlInputElement>()
            .files()
            .unwrap(),
    );
    open_file(files.first().unwrap()).await;
}

async fn open_file(file: &File) {
    show_output(&format!(
        "{}: {} bytes, {}",
        file.name(),
        file.size(),
        file.raw_mime_type()
    ));
    let text = read_as_text(&file).await.unwrap();
    match from_str(&text) {
        Ok(song) => {
            show_error("");
            STATE.lock().unwrap().songs.push(song);
            update_song_list();
        }
        Err(e) => show_error(&e.to_string()),
    }
}

fn playlist_entry_selected(_event: Event) {
    let selected_index = document()
        .get_element_by_id("playlist")
        .expect("Couldn't find playlist")
        .unchecked_into::<HtmlSelectElement>()
        .selected_index();
    let Ok(selected_index) = usize::try_from(selected_index) else {
        return;
    };
    let state = STATE.lock().unwrap();
    let slide = &state.slides()[selected_index];
    match slide {
        Slide::Lyrics {
            song_index,
            lyric_entry_index,
            lines_index,
        } => {
            let song = &state.songs[*song_index];
            show_song_page(song, *lyric_entry_index, *lines_index);
        }
        Slide::Text(text) => {
            show_text_page(text);
        }
        Slide::SongStart { .. } => {}
    }
}

/// Updates the song list in the UI with the list of songs in the model.
fn update_song_list() {
    let mut html = String::new();
    for (i, song) in STATE.lock().unwrap().songs.iter().enumerate() {
        writeln!(
            &mut html,
            "<option value=\"{i}\">{}</option>",
            title_for_song(song),
        )
        .unwrap();
    }
    document()
        .get_element_by_id("song_list")
        .expect("Couldn't find song_list")
        .set_inner_html(&html);
}

fn update_playlist() {
    let mut html = String::new();
    let state = STATE.lock().unwrap();
    for slide in state.slides() {
        match slide {
            Slide::SongStart { song_index } => {
                let title = title_for_song(&state.songs[song_index]);
                writeln!(&mut html, "<option disabled>{title}</option>").unwrap();
            }
            Slide::Lyrics {
                song_index,
                lyric_entry_index,
                lines_index,
            } => {
                let song = &state.songs[song_index];
                if lines_index == 0 {
                    writeln!(
                        &mut html,
                        "<option>- {}</option>",
                        song.lyrics.lyrics[lyric_entry_index].name(),
                    )
                    .unwrap();
                } else {
                    writeln!(&mut html, "<option>{lines_index}</option>").unwrap();
                }
            }
            Slide::Text(text) => {
                writeln!(&mut html, "<option>{text}</option>").unwrap();
            }
        }
    }
    document()
        .get_element_by_id("playlist")
        .expect("Couldn't find playlist")
        .set_inner_html(&html);
}

fn show_text_page(text: &str) {
    let song_element = document()
        .get_element_by_id("song")
        .expect("Couldn't find song element");
    song_element.set_inner_html(&format!("<p>{text}</p>"));
}

fn show_song_page(song: &Song, lyric_entry_index: usize, lines_index: usize) {
    let mut song_html = String::new();
    writeln!(&mut song_html, "<h1>{}</h1>", title_for_song(song)).unwrap();

    let item = &song.lyrics.lyrics[lyric_entry_index];
    match item {
        LyricEntry::Verse { name, lines, .. } => {
            writeln!(&mut song_html, "<h2>{name}</h2>").unwrap();
            writeln!(&mut song_html, "<div class=\"verse\">").unwrap();
            let line = &lines[lines_index];

            writeln!(&mut song_html, "<p>").unwrap();
            if let Some(part) = &line.part {
                writeln!(&mut song_html, "<em>({part})</em><br/>").unwrap();
            }
            for simple_line in &simplify_contents(&line.contents) {
                writeln!(&mut song_html, "{simple_line}<br/>").unwrap();
            }
            if let Some(repeat) = line.repeat {
                writeln!(&mut song_html, "<strong>x{repeat}</strong><br/>").unwrap();
            }
            writeln!(&mut song_html, "</p>").unwrap();

            writeln!(&mut song_html, "</div>").unwrap();
        }
        LyricEntry::Instrument { name, .. } => {
            writeln!(&mut song_html, "<p>(instrumental {name})</p>").unwrap()
        }
    }

    let song_element = document()
        .get_element_by_id("song")
        .expect("Couldn't find song element");
    song_element.set_inner_html(&song_html);
}

fn show_output(text: &str) {
    let error_element = document()
        .get_element_by_id("output")
        .expect("Couldn't find output element");
    error_element.set_text_content(Some(text));
}

fn show_error(error: &str) {
    let error_element = document()
        .get_element_by_id("error")
        .expect("Couldn't find error element");
    error_element.set_text_content(Some(error));
}
