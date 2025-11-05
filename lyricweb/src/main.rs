// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod model;

use crate::model::{PlaylistEntry, Slide, State, title_for_song};
use gloo_file::{File, FileList, futures::read_as_text};
use leptos::{ev::Targeted, prelude::*, task::spawn_local};
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};
use quick_xml::de::from_str;
use std::{
    fmt::Write,
    hash::{DefaultHasher, Hash, Hasher},
};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlDivElement, HtmlInputElement, HtmlSelectElement, SubmitEvent};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let text_entry = NodeRef::new();
    let song_list = NodeRef::new();
    let song = NodeRef::new();

    let (state, write_state) = signal(State::new());
    let (output, write_output) = signal(None);
    let (error, write_error) = signal(None);

    view! {
        <h1>"Lyricweb"</h1>
        <form>
        <input type="file" on:change:target=move |event| spawn_local(file_changed(event, write_state, write_output, write_error)) />
        </form>
        <p id="output">{ output }</p>
        <p id="error">{ error }</p>
        <form on:submit=move |event| add_song_to_playlist(event, song_list.get().unwrap(), write_state, write_output)>
        <select size="10" node_ref=song_list>
            <For
                // TODO: Avoid clone and collect
                each=move || { state.read().songs.iter().cloned().enumerate().collect::<Vec<_>>() }
                // TODO: Use a better key
                key=|(i, _song)| *i
                children=move |(i, song)| {
                    view! {
                        <option value=i>{ move || title_for_song(&song).to_owned() }</option>
                    }
                }
            />
        </select>
        <input type="submit" value="Add to playlist" />
        </form>
        <form on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap(), write_state)>
        <input type="text" node_ref=text_entry />
        <input type="submit" value="Add to playlist" />
        </form>
        <form>
        <select size="20" on:change:target=move |event| playlist_entry_selected(event, song.get().unwrap(), state)>
            <For
                each=move|| { state.read().slides().into_iter().enumerate() }
                key=|slide| {
                    // Include both index and value in key, as a slide might be repeated.
                    let mut hasher = DefaultHasher::new();
                    slide.hash(&mut hasher);
                    hasher.finish()
                }
                children=move |(_, slide)| {
                    match slide {
                        Slide::SongStart { song_index } => {
                            view! {
                                <option disabled>{ move || title_for_song(&state.read().songs[song_index]).to_owned() }</option>
                            }.into_any()
                        }
                        Slide::Lyrics {
                            song_index,
                            lyric_entry_index,
                            lines_index,
                        } => {
                            let state = state.read();
                            let song = &state.songs[song_index];
                            let lyric_entry = &song.lyrics.lyrics[lyric_entry_index];

                            let first_line = if let LyricEntry::Verse { lines, .. } = lyric_entry {
                                simplify_contents(&lines[lines_index].contents)
                                    .into_iter()
                                    .next()
                            } else {
                                None
                            };

                            view! {
                                <option>{
                                    if lines_index == 0 {
                                        format!("- {}", lyric_entry.name())
                                    } else {
                                        "...".to_string()
                                    }
                                }{
                                    first_line.map(|first_line| format!(": {first_line}"))
                                }</option>
                            }.into_any()
                        }
                        Slide::Text(text) => {
                            view! {
                                <option>{ text }</option>
                            }.into_any()
                        }
                    }
                }
            />
        </select>
        </form>
        <div node_ref=song></div>
    }
}

fn add_song_to_playlist(
    event: SubmitEvent,
    song_list: HtmlSelectElement,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
) {
    event.prevent_default();

    let selected = song_list.selected_index();
    write_output.set(Some(format!("selected: {selected}")));
    if selected >= 0 {
        write_state.update(|state| {
            state.playlist.push(PlaylistEntry::Song {
                song_index: selected.try_into().unwrap(),
            })
        });
    }
}

fn add_text_to_playlist(
    event: SubmitEvent,
    text_entry: HtmlInputElement,
    write_state: WriteSignal<State>,
) {
    event.prevent_default();

    let text = text_entry.value();
    write_state.update(|state| state.playlist.push(PlaylistEntry::Text(text)));
}

async fn file_changed(
    event: Targeted<Event, HtmlInputElement>,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
    write_error: WriteSignal<Option<String>>,
) {
    let files = FileList::from(event.target().files().unwrap());
    open_file(
        files.first().unwrap(),
        write_state,
        write_output,
        write_error,
    )
    .await;
}

async fn open_file(
    file: &File,
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
    match from_str(&text) {
        Ok(song) => {
            write_error.set(None);
            write_state.update(|state| state.songs.push(song));
        }
        Err(e) => write_error.set(Some(e.to_string())),
    }
}

fn playlist_entry_selected(
    event: Targeted<Event, HtmlSelectElement>,
    song_element: HtmlDivElement,
    state: ReadSignal<State>,
) {
    let selected_index = event.target().selected_index();
    let Ok(selected_index) = usize::try_from(selected_index) else {
        return;
    };
    let state = state.read();
    let slide = &state.slides()[selected_index];
    match slide {
        Slide::Lyrics {
            song_index,
            lyric_entry_index,
            lines_index,
        } => {
            let song = &state.songs[*song_index];
            show_song_page(song_element, song, *lyric_entry_index, *lines_index);
        }
        Slide::Text(text) => {
            show_text_page(song_element, text);
        }
        Slide::SongStart { .. } => {}
    }
}

fn show_text_page(song_element: HtmlDivElement, text: &str) {
    song_element.set_inner_html(&format!("<p>{text}</p>"));
}

fn show_song_page(
    song_element: HtmlDivElement,
    song: &Song,
    lyric_entry_index: usize,
    lines_index: usize,
) {
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

    song_element.set_inner_html(&song_html);
}
