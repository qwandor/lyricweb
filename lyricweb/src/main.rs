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
use std::hash::{DefaultHasher, Hash, Hasher};
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, SubmitEvent};

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let text_entry = NodeRef::new();

    let state = RwSignal::new(State::new());
    let (output, write_output) = signal(None);
    let (error, write_error) = signal(None);
    let (current_slide, write_current_slide) = signal(None);

    view! {
        <h1>"Lyricweb"</h1>
        <form>
        <input type="file" on:change:target=move |event| spawn_local(file_changed(event, state.write_only(), write_output, write_error)) />
        </form>
        <p id="output">{ output }</p>
        <p id="error">{ error }</p>
        <SongList state write_output/>
        <form on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap(), state.write_only())>
        <input type="text" node_ref=text_entry />
        <input type="submit" value="Add to playlist" />
        </form>
        <Playlist state write_current_slide/>
        <CurrentSlide state=state.read_only() current_slide/>
    }
}

#[component]
fn SongList(state: RwSignal<State>, write_output: WriteSignal<Option<String>>) -> impl IntoView {
    let song_list = NodeRef::new();

    view! {
        <form on:submit=move |event| add_song_to_playlist(event, song_list.get().unwrap(), state.write_only(), write_output)>
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
    }
}

#[component]
fn Playlist(
    state: RwSignal<State>,
    write_current_slide: WriteSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <form>
        <select size="20"
            on:change:target=move |event| {
                if let Ok(selected_index) = usize::try_from(event.target().selected_index()) {
                    write_current_slide.set(Some(selected_index))
                }
            }>
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
    }
}

#[component]
fn CurrentSlide(
    state: ReadSignal<State>,
    current_slide: ReadSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        { move || {
            let state = state.read();
            let slide = &state.slides()[current_slide.get()?];
            match slide {
                Slide::SongStart { .. } => None,
                Slide::Lyrics {
                    song_index,
                    lyric_entry_index,
                    lines_index,
                } => {
                    let song = &state.songs[*song_index];
                    Some(song_page(song, *lyric_entry_index, *lines_index).into_any())
                }
                Slide::Text(text) => Some(text_page(text).into_any()),
            }
        } }
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

fn text_page(text: &str) -> impl IntoView {
    view! {
        <p>{text}</p>
    }
}

fn song_page(song: &Song, lyric_entry_index: usize, lines_index: usize) -> impl IntoView {
    let item = &song.lyrics.lyrics[lyric_entry_index];

    view! {
        <h1>{ title_for_song(song) }</h1>
        {
            match item {
                LyricEntry::Verse { name, lines, .. } => {
                    let line = &lines[lines_index];
                    view! {
                        <h2>{name.as_str()}</h2>
                        <div class="verse">
                        <p>
                        { line.part.as_ref().map(|part| view! { <em>"(" {part.as_str()} ")"</em><br/> }) }
                        { simplify_contents(&line.contents).into_iter().map(|simple_line| view! { {simple_line} <br/> } ).collect::<Vec<_>>() }
                        { line.repeat.map(|repeat| view! { <strong>"x" {repeat}</strong><br/> } ) }
                        </p>
                        </div>
                    }.into_any()
                }
                LyricEntry::Instrument { name, .. } => {
                    view! { <p>"(instrumental " {name.as_str()} ")"</p> }.into_any()
                }
            }
        }
    }
}
