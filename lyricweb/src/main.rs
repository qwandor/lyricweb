// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod model;

use crate::model::{PlaylistEntry, Slide, State, title_for_song};
use gloo_file::{File, FileList, futures::read_as_text};
use gloo_utils::document;
use leptos::{ev::Targeted, prelude::*, task::spawn_local};
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};
use quick_xml::de::from_str;
use std::{fmt::Write, sync::Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, EventTarget, HtmlInputElement, HtmlSelectElement, SubmitEvent};

static STATE: Mutex<State> = Mutex::new(State::new());

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(App);

    add_listener_by_id("song_list_form", "submit", add_song_to_playlist);
    add_listener_by_id("playlist", "change", playlist_entry_selected);
}

#[component]
fn App() -> impl IntoView {
    let text_entry = NodeRef::new();

    view! {
        <h1>"Lyricweb"</h1>
        <form>
        <input type="file" on:change:target=move |event| spawn_local(file_changed2(event)) />
        </form>
        <p id="output2"></p>
        <p id="error2"></p>
        <form id="song_list_form2">
        <select id="song_list2" size="10">
        </select>
        <input type="submit" value="Add to playlist" />
        </form>
        <form on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap())>
        <input type="text" node_ref=text_entry />
        <input type="submit" value="Add to playlist" />
        </form>
        <form>
        <select id="playlist2" size="20"></select>
        </form>
        <div id="song2"></div>
    }
}

fn get_element_by_id(id: &str) -> Element {
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Failed to find element {id}"))
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

fn add_text_to_playlist(event: SubmitEvent, text_entry: HtmlInputElement) {
    event.prevent_default();

    let text = text_entry.value();
    STATE
        .lock()
        .unwrap()
        .playlist
        .push(PlaylistEntry::Text(text));
    update_playlist();
}

async fn file_changed2(event: Targeted<Event, HtmlInputElement>) {
    let files = FileList::from(event.target().files().unwrap());
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
                let lyric_entry = &song.lyrics.lyrics[lyric_entry_index];

                let first_line = if let LyricEntry::Verse { lines, .. } = lyric_entry {
                    simplify_contents(&lines[lines_index].contents)
                        .into_iter()
                        .next()
                } else {
                    None
                };

                write!(&mut html, "<option>").unwrap();
                if lines_index == 0 {
                    write!(&mut html, "- {}", lyric_entry.name(),).unwrap();
                } else {
                    write!(&mut html, "...").unwrap();
                }
                if let Some(first_line) = first_line {
                    write!(&mut html, ": {first_line}",).unwrap();
                }
                writeln!(&mut html, "</option>").unwrap();
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
