// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{PlaylistEntry, State, title_for_song};
use leptos::prelude::*;
use web_sys::{HtmlSelectElement, SubmitEvent};

/// List of all available songs.
#[component]
pub fn SongList(
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
