// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{
    Playlist, Slide, SlideIndex, State,
    helpers::{first_line, title_with_songbook},
};
use leptos::prelude::*;
use web_sys::{HtmlInputElement, SubmitEvent};

/// Playlist of songs and other items to be presented.
#[component]
pub fn Playlist(
    state: Signal<State>,
    write_state: WriteSignal<State>,
    current_playlist: Signal<Option<u32>>,
    write_current_playlist: WriteSignal<Option<u32>>,
    current_slide: Signal<Option<SlideIndex>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
) -> impl IntoView {
    let no_current_playlist = move || current_playlist.get().is_none();
    let no_current_slide = move || current_slide.get().is_none();

    let playlist_name = NodeRef::new();

    view! {
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
            <form class="wide" on:submit=move |event| rename_playlist(event, playlist_name.get().unwrap(), current_playlist, write_state)>
                <input type="text" node_ref=playlist_name minlength="1" size="10"
                    prop:value=move || current_playlist.get().and_then(|playlist_id| Some(state.get().playlists.get(&playlist_id)?.name.clone())).unwrap_or_default() />
                <input type="submit" value="Rename" disabled=no_current_playlist />
                <input type="button" value="Duplicate" disabled=no_current_playlist on:click=move |_| duplicate_playlist(playlist_name.get().unwrap(), write_state, current_playlist, write_current_playlist) />
            </form>
        </div>
        <form class="tall">
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
                                <option value={slide_index.to_string()}>{ title_with_songbook(&state.songs[&song_id]).to_owned() }</option>
                            }.into_any()
                        }
                        Slide::Lyrics {
                            song_id,
                            lyric_entry_index,
                            lines_index,
                            ..
                        } => {
                            let song = &state.songs[&song_id];
                            let lyric_entry = &song.lyrics.lyrics[lyric_entry_index];
                            let first_line = first_line(song, lyric_entry_index, lines_index);

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

fn rename_playlist(
    event: SubmitEvent,
    text_entry: HtmlInputElement,
    current_playlist: Signal<Option<u32>>,
    write_state: WriteSignal<State>,
) {
    event.prevent_default();

    let Some(current_playlist) = current_playlist.get() else {
        return;
    };

    let new_name = text_entry.value();
    write_state.update(|state| state.playlists.get_mut(&current_playlist).unwrap().name = new_name);
}

/// Creates a new playlist and switches to it.
fn new_playlist(write_state: WriteSignal<State>, write_current_playlist: WriteSignal<Option<u32>>) {
    let mut new_playlist_id = 0;
    write_state.update(|state| new_playlist_id = state.add_playlist(Playlist::new("New")));
    write_current_playlist.set(Some(new_playlist_id));
}

/// Makes a copy of the current playlist with a new name and switches to it.
fn duplicate_playlist(
    name: HtmlInputElement,
    write_state: WriteSignal<State>,
    current_playlist: Signal<Option<u32>>,
    write_current_playlist: WriteSignal<Option<u32>>,
) {
    let Some(playlist_id) = current_playlist.get() else {
        return;
    };

    let mut state = write_state.write();
    let Some(playlist) = state.playlists.get(&playlist_id) else {
        return;
    };
    let mut playlist = playlist.clone();
    playlist.name = name.value();
    let new_playlist_id = state.add_playlist(playlist);
    drop(state);
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
        write_current_playlist.set(
            state
                .playlists
                .first_key_value()
                .map(|(&first_id, _)| first_id),
        );
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
