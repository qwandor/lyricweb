// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{State, lyrics_as_text, title_for_song};
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
        let authors = &song.properties.authors.authors;
        let verse_order = song
            .properties
            .verse_order
            .as_deref()
            .unwrap_or_default()
            .to_owned();
        let lyrics_text = lyrics_as_text(&song);

        let title = NodeRef::new();
        let verseorder = NodeRef::new();
        let lyrics = NodeRef::new();
        Some(view! {
            <h2>"Edit song"</h2>
            <form class="tall" on:submit=move |event| save_song(event, write_state, song_id, title.get().unwrap())>
                <table>
                    <tr>
                        <td><label for="title">Title</label></td>
                        <td><input type="text" id="title" node_ref=title prop:value=title_for_song(&song).to_owned()/></td>
                    </tr>
                    <tr>
                        <td><label for="author">Author</label></td>
                        <td><input type="text" id="author" prop:value=authors[0].name.to_owned()/></td>
                    </tr>
                    <tr>
                        <td><label for="verseorder">Verse order</label></td>
                        <td><input type="text" id="verseorder" node_ref=verseorder prop:value=verse_order/></td>
                    </tr>
                </table>
                <textarea class="tall" node_ref=lyrics prop:value=lyrics_text></textarea>
                <div class="button-row">
                    <input type="submit" value="Save"/>
                    <input type="button" value="Close" on:click=move |_| write_edit_song.set(None) />
                </div>
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
