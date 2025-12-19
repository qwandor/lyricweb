// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{
    PlaylistEntry, State,
    helpers::{first_line, song_matches_filter, title_for_song},
};
use leptos::prelude::*;
use web_sys::SubmitEvent;

/// List of all available songs.
#[component]
pub fn SongList(
    state: Signal<State>,
    write_state: WriteSignal<State>,
    current_playlist: Signal<Option<u32>>,
    write_edit_song: WriteSignal<Option<u32>>,
) -> impl IntoView {
    let no_current_playlist = move || current_playlist.get().is_none();
    let (selected_song, write_selected_song) = signal(None);
    let (filter, write_filter) = signal(String::new());

    view! {
        <form class="tall" on:submit=move |event| add_song_to_playlist(event, selected_song, current_playlist, write_state)>
            <input type="text" placeholder="Search" on:input:target=move |event| write_filter.set(event.target().value()) />
            <select size="5" id="song-list" on:change:target=move |event| {
                write_selected_song.set(event.target().value().parse().ok());
            }>
                {move || {
                    let state = state.read();
                    state.songs_by_title().into_iter().filter(|(_, song)| song_matches_filter(song, &filter.read())).map(|(id, song)| {
                        view! {
                            <option value={id.to_string()}>{title_for_song(&song).to_owned()}</option>
                        }
                    }).collect::<Vec<_>>()
                }}
            </select>
            <SongInfo state selected_song />
            <div class="button-row">
                <input type="button" value="Remove" on:click=move |_| remove_from_song_list(selected_song, write_state) />
                <input type="button" value="Edit" on:click=move |_| write_edit_song.set(selected_song.get()) />
                <input type="submit" value="Add to playlist" disabled=no_current_playlist />
            </div>
        </form>
    }
}

/// Information about a particular song.
#[component]
fn SongInfo(state: Signal<State>, selected_song: ReadSignal<Option<u32>>) -> impl IntoView {
    move || {
        let state = state.read();
        let song_id = selected_song.get()?;
        let song = state.songs.get(&song_id)?;
        Some(view! {
            <div>
            <h2>{title_for_song(&song).to_owned()}</h2>
            <p>
                "Author: "
                {song.properties.authors.authors.iter().map(|author| {
                    let author_name = &author.name;
                    if let Some(author_type) = &author.author_type {
                        format!("{author_name} ({author_type})")
                    } else {
                        format!("{author_name}")
                    }
                }).collect::<Vec<_>>().join(", ") }
                <br/>
                "First line: " {first_line(&song, 0, 0)}
            </p>
            </div>
        })
    }
}

/// Removes the selected song from the song database.
fn remove_from_song_list(selected_song: ReadSignal<Option<u32>>, write_state: WriteSignal<State>) {
    let Some(song_id) = selected_song.get() else {
        return;
    };

    write_state.update(|state| {
        state.remove_song(song_id);
    })
}

fn add_song_to_playlist(
    event: SubmitEvent,
    selected_song: ReadSignal<Option<u32>>,
    current_playlist: Signal<Option<u32>>,
    write_state: WriteSignal<State>,
) {
    event.prevent_default();

    let Some(song_id) = selected_song.get() else {
        return;
    };
    let Some(current_playlist) = current_playlist.get() else {
        return;
    };

    write_state.update(|state| {
        state
            .playlists
            .get_mut(&current_playlist)
            .unwrap()
            .entries
            .push(PlaylistEntry::Song { song_id })
    });
}
