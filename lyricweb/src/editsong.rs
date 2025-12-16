// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{State, title_for_song};
use leptos::prelude::*;
use web_sys::{HtmlInputElement, SubmitEvent};

#[component]
pub fn EditSong(
    state: Signal<State>,
    write_state: WriteSignal<State>,
    edit_song: ReadSignal<Option<u32>>,
    write_edit_song: WriteSignal<Option<u32>>,
) -> impl IntoView {
    move || {
        let state = state.read();
        let song_id = edit_song.get()?;
        let song = state.songs.get(&song_id)?;
        let title = NodeRef::new();
        Some(view! {
            <h2>"Edit song"</h2>
            <form on:submit=move |event| save_song(event, write_state, song_id, title.get().unwrap())>
                <table>
                    <tr>
                        <td>Title</td>
                        <td><input type="text" node_ref=title prop:value=title_for_song(&song).to_owned()/></td>
                    </tr>
                </table>
                <input type="submit" value="Save"/>
                <input type="button" value="Close" on:click=move |_| write_edit_song.set(None) />
            </form>
        })
    }
}

fn save_song(
    event: SubmitEvent,
    write_state: WriteSignal<State>,
    song_id: u32,
    title: HtmlInputElement,
) {
    event.prevent_default();
    let title = title.value();

    write_state.update(|state| {
        let Some(song) = state.songs.get_mut(&song_id) else {
            return;
        };

        song.properties.titles.titles[0].title = title;
    });
}
