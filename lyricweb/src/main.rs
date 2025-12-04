// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod model;

use crate::model::{PlaylistEntry, Slide, SlideIndex, State, title_for_song};
use gloo_file::{File, FileList, futures::read_as_text};
use leptos::{
    ev::Targeted,
    prelude::*,
    server::codee::string::{FromToStringCodec, JsonSerdeCodec, OptionCodec},
    task::spawn_local,
};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::query_signal,
    path,
};
use leptos_use::storage::use_local_storage;
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};
use quick_xml::de::from_str;
use std::cell::RefCell;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, SubmitEvent, Window};

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (state, write_state, _) = use_local_storage::<_, JsonSerdeCodec>("state");
    let (current_slide, write_current_slide, _) =
        use_local_storage::<_, OptionCodec<FromToStringCodec>>("current_slide");

    view! {
        <Router>
            <Routes fallback=|| "Not found">
                <Route path=path!("*any") view=move || if query_signal("present").0.get().unwrap_or_default() {
                    view! {
                        <CurrentSlide state current_slide/>
                    }.into_any()
                } else {
                    view! {
                        <Controller state write_state current_slide write_current_slide/>
                    }.into_any()
                }
                />
            </Routes>
        </Router>
    }
}

/// The main view for controlling the presentation.
#[component]
fn Controller(
    state: Signal<State>,
    write_state: WriteSignal<State>,
    current_slide: Signal<Option<SlideIndex>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
) -> impl IntoView {
    let text_entry = NodeRef::new();

    let (current_playlist, write_current_playlist, _) =
        use_local_storage::<_, OptionCodec<FromToStringCodec>>("current_playlist");
    let no_current_playlist = move || current_playlist.get().is_none();

    let (output, write_output) = signal(None);
    let (error, write_error) = signal(None);

    let presentation_window = RefCell::new(None);

    view! {
        <div id="controller">
        <div class="column">
        <h1>"Lyricweb"</h1>
        <form>
        <input type="file" on:change:target=move |event| spawn_local(file_changed(event, write_state, write_output, write_error)) />
        </form>
        <div>
        <p id="output">{ output }</p>
        <p id="error">{ error }</p>
        </div>
        <SongList state write_state current_playlist write_output/>
        <form on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap(), current_playlist, write_state)>
        <input type="text" node_ref=text_entry />
        <input type="submit" value="Add to playlist" disabled=no_current_playlist />
        </form>
        </div>
        <div class="column">
        <Playlist state write_state current_playlist write_current_playlist current_slide write_current_slide/>
        </div>
        <div class="column">
        <form>
        <input type="button" value="Present" on:click=move |_| open_presentation(&mut presentation_window.borrow_mut())/>
        </form>
        <div class="preview">
        <CurrentSlide state current_slide/>
        </div>
        </div>
        </div>
    }
}

/// Opens a new window to show the presentation.
fn open_presentation(presentation_window: &mut Option<Window>) {
    // If there's already a presentation window open, close it.
    if let Some(presentation_window) = presentation_window {
        presentation_window.close().unwrap();
    }

    let new_presentation_window = window()
        .open_with_url_and_target_and_features(&"?present=true", &"", &"popup=true")
        .unwrap()
        .unwrap();

    *presentation_window = Some(new_presentation_window);
}

#[component]
fn SongList(
    state: Signal<State>,
    write_state: WriteSignal<State>,
    current_playlist: Signal<Option<u32>>,
    write_output: WriteSignal<Option<String>>,
) -> impl IntoView {
    let song_list = NodeRef::new();
    let no_current_playlist = move || current_playlist.get().is_none();

    view! {
        <form class="tall" on:submit=move |event| add_song_to_playlist(event, song_list.get().unwrap(), current_playlist, write_state, write_output)>
            <select size="5" id="song-list" node_ref=song_list>
                {move || {
                    let state = state.read();
                    state.songs_by_title().into_iter().map(|(id, song)| {
                        view! {
                            <option value={id.to_string()}>{title_for_song(&song).to_owned()}</option>
                        }
                    }).collect::<Vec<_>>()
                }}
            </select>
            <div class="button-row">
            <input type="button" value="Remove" on:click=move |_| remove_from_song_list(song_list.get().unwrap(), write_state) />
            <input type="submit" value="Add to playlist" disabled=no_current_playlist />
            </div>
        </form>
    }
}

#[component]
fn Playlist(
    state: Signal<State>,
    write_state: WriteSignal<State>,
    current_playlist: Signal<Option<u32>>,
    write_current_playlist: WriteSignal<Option<u32>>,
    current_slide: Signal<Option<SlideIndex>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
) -> impl IntoView {
    let no_current_playlist = move || current_playlist.get().is_none();
    let no_current_slide = move || current_slide.get().is_none();

    view! {
        <h2>{move || Some(state.get().playlists.get(&current_playlist.get()?)?.name.clone())}</h2>
        <form class="tall">
        <div class="button-row">
        <select on:change:target=move |event| if let Ok(playlist_id) = event.target().value().parse() {
            // TODO: What should this do about the current slide?
            write_current_playlist.set(Some(playlist_id));
        }
        prop:value=move || current_playlist.get().map(|playlist_id| playlist_id.to_string())>
        {move || {
            let state = state.read();
            state.playlists.iter().map(|(playlist_id, playlist)| {
                view! {
                    <option value={playlist_id.to_string()}>{playlist.name.clone()}</option>
                }
            }).collect::<Vec<_>>()
        }}
        </select>
        <input type="button" value="New" on:click=move |_| new_playlist(write_state, write_current_playlist)/>
        <input type="button" value="Delete" disabled=no_current_playlist on:click=move |_| delete_playlist(write_state, current_playlist, write_current_playlist, write_current_slide)/>
        </div>
        <select size="5" id="playlist" disabled=no_current_playlist
            on:change:target=move |event| {
                if let Ok(slide_index) = event.target().value().parse() {
                    write_current_slide.set(Some(slide_index));
                }
            }
            prop:value=move || current_slide.get().map(|index| index.to_string())>
            {move || {
                let state = state.read();
                let Some(current_playlist) = current_playlist.get() else {
                    return Vec::new();
                };
                state.slides(current_playlist).into_iter().map(|(slide_index, slide)| {
                    match slide {
                        Slide::SongStart { song_id } => {
                            view! {
                                <option disabled value={slide_index.to_string()}>{ title_for_song(&state.songs[&song_id]).to_owned() }</option>
                            }.into_any()
                        }
                        Slide::Lyrics {
                            song_id,
                            lyric_entry_index,
                            lines_index,
                        } => {
                            let song = &state.songs[&song_id];
                            let lyric_entry = &song.lyrics.lyrics[lyric_entry_index];

                            let first_line = if let LyricEntry::Verse { lines, .. } = lyric_entry {
                                simplify_contents(&lines[lines_index].contents)
                                    .into_iter()
                                    .next()
                            } else {
                                None
                            };

                            view! {
                                <option value={slide_index.to_string()}>{
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
                                <option value={slide_index.to_string()}>{ text }</option>
                            }.into_any()
                        }
                    }
                }).collect::<Vec<_>>()
            }}
        </select>
        <div class="button-row">
        <input type="button" value="Remove" disabled=no_current_slide on:click=move |_| remove_from_playlist(write_state, current_slide, write_current_slide)/>
        <input type="button" value="Move up" disabled=no_current_slide on:click=move |_| move_in_playlist(write_state, current_slide, write_current_slide, -1)/>
        <input type="button" value="Move down" disabled=no_current_slide on:click=move |_| move_in_playlist(write_state, current_slide, write_current_slide, 1)/>
        </div>
        </form>
    }
}

/// Creates a new playlist and switches to it.
fn new_playlist(write_state: WriteSignal<State>, write_current_playlist: WriteSignal<Option<u32>>) {
    let mut new_playlist_id = 0;
    write_state.update(|state| new_playlist_id = state.add_playlist("New"));
    write_current_playlist.set(Some(new_playlist_id));
}

/// Deletes the current playlist.
fn delete_playlist(
    write_state: WriteSignal<State>,
    current_playlist: Signal<Option<u32>>,
    write_current_playlist: WriteSignal<Option<u32>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
) {
    let Some(playlist_id) = current_playlist.get() else {
        return;
    };

    write_current_slide.set(None);
    write_state.update(|state| {
        state.playlists.remove(&playlist_id);
        if let Some((&first_id, _)) = state.playlists.first_key_value() {
            write_current_playlist.set(Some(first_id));
        } else {
            write_current_playlist.set(None);
        }
    });
}

/// Removes the current slide's entry from the playlist.
fn remove_from_playlist(
    write_state: WriteSignal<State>,
    current_slide: Signal<Option<SlideIndex>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
) {
    if let Some(mut current_slide) = current_slide.get() {
        write_state.update(|state| {
            let playlist = state.playlists.get_mut(&current_slide.playlist_id).unwrap();
            playlist.entries.remove(current_slide.entry_index);

            if playlist.entries.is_empty() {
                write_current_slide.set(None);
            } else {
                // Ensure that current_slide is still within range.
                current_slide.page_index = 0;
                if current_slide.entry_index >= playlist.entries.len() {
                    current_slide.entry_index -= 1;
                }
                write_current_slide.set(Some(current_slide));
            }
        });
    }
}

/// Moves the current slide's entry up or down in the playlist.
fn move_in_playlist(
    write_state: WriteSignal<State>,
    current_slide: Signal<Option<SlideIndex>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
    offset: isize,
) {
    if let Some(current_slide) = current_slide.get() {
        let mut moved = false;
        write_state.update(|state| {
            moved = state
                .playlists
                .get_mut(&current_slide.playlist_id)
                .unwrap()
                .move_entry_index(current_slide.entry_index, offset)
        });
        if moved {
            write_current_slide.update(|current_slide| {
                if let Some(current_slide) = current_slide {
                    current_slide.entry_index = current_slide
                        .entry_index
                        .checked_add_signed(offset)
                        .unwrap();
                }
            });
        }
    }
}

#[component]
fn CurrentSlide(state: Signal<State>, current_slide: Signal<Option<SlideIndex>>) -> impl IntoView {
    view! {
        <div class="slide">
        { move || {
            let state = state.read();
            let slide = &state.slide(current_slide.get()?)?;
            match slide {
                Slide::SongStart { .. } => None,
                Slide::Lyrics {
                    song_id,
                    lyric_entry_index,
                    lines_index,
                } => {
                    let song = &state.songs[song_id];
                    Some(song_page(song, *lyric_entry_index, *lines_index).into_any())
                }
                Slide::Text(text) => Some(text_page(text).into_any()),
            }
        } }
        </div>
    }
}

/// Removes the selected song from the song database.
fn remove_from_song_list(song_list: HtmlSelectElement, write_state: WriteSignal<State>) {
    let Ok(song_id) = song_list.value().parse() else {
        return;
    };

    write_state.update(|state| {
        state.remove_song(song_id);
    })
}

fn add_song_to_playlist(
    event: SubmitEvent,
    song_list: HtmlSelectElement,
    current_playlist: Signal<Option<u32>>,
    write_state: WriteSignal<State>,
    write_output: WriteSignal<Option<String>>,
) {
    event.prevent_default();

    let Ok(song_id) = song_list.value().parse() else {
        return;
    };
    let Some(current_playlist) = current_playlist.get() else {
        return;
    };

    write_output.set(Some(format!("song_id: {song_id}")));
    write_state.update(|state| {
        state
            .playlists
            .get_mut(&current_playlist)
            .unwrap()
            .entries
            .push(PlaylistEntry::Song { song_id })
    });
}

fn add_text_to_playlist(
    event: SubmitEvent,
    text_entry: HtmlInputElement,
    current_playlist: Signal<Option<u32>>,
    write_state: WriteSignal<State>,
) {
    event.prevent_default();

    let Some(current_playlist) = current_playlist.get() else {
        return;
    };

    let text = text_entry.value();
    write_state.update(|state| {
        state
            .playlists
            .get_mut(&current_playlist)
            .unwrap()
            .entries
            .push(PlaylistEntry::Text(text))
    });
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
            write_state.update(|state| state.add_song(song));
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
