// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::{
    model::{
        State,
        helpers::{
            authors_as_string, lyrics_as_text, set_authors_from_string, set_lyrics_from_text,
            set_songbooks_from_string, songbook_entries_as_string, title_for_song,
        },
        slide::SlideContent,
    },
    slide::Slide,
};
use leptos::prelude::*;
use web_sys::{HtmlInputElement, HtmlTextAreaElement, SubmitEvent};

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
        let verse_order = song
            .properties
            .verse_order
            .as_deref()
            .unwrap_or_default()
            .to_owned();
        let lyrics_text = lyrics_as_text(&song);

        let title = NodeRef::new();
        let authors = NodeRef::new();
        let songbook_entries = NodeRef::new();
        let verseorder = NodeRef::new();
        let lyrics = NodeRef::new();
        Some(view! {
            <h2>"Edit song"</h2>
            <form class="tall"
                on:submit=move |event| save_song(event, write_state, song_id, title.get().unwrap(), authors.get().unwrap(), songbook_entries.get().unwrap(), verseorder.get().unwrap(), lyrics.get().unwrap())>
                <table>
                    <tr>
                        <td><label for="title">Title</label></td>
                        <td><input type="text" id="title" node_ref=title prop:value=title_for_song(&song).to_owned()/></td>
                    </tr>
                    <tr>
                        <td><label for="author">Author</label></td>
                        <td><input type="text" id="author" node_ref=authors prop:value=authors_as_string(&song)/></td>
                    </tr>
                    <tr>
                        <td><label for="songbook_entries">Songbook entries</label></td>
                        <td><input type="text" id="songbook_entries" node_ref=songbook_entries prop:value=songbook_entries_as_string(&song)/></td>
                    </tr>
                    <tr>
                        <td><label for="verseorder">Verse order</label></td>
                        <td><input type="text" id="verseorder" node_ref=verseorder prop:value=verse_order/></td>
                    </tr>
                </table>
                <textarea node_ref=lyrics prop:value=lyrics_text></textarea>
                <div class="button-row">
                    <input type="submit" value="Save"/>
                    <input type="button" value="Close" on:click=move |_| write_edit_song.set(None) />
                </div>
            </form>
        })
    }
}

#[component]
pub fn PreviewSlides(state: Signal<State>, song_id: ReadSignal<Option<u32>>) -> impl IntoView {
    move || {
        let state = state.read();
        let song_id = song_id.get()?;
        let slides = state.slides_for_song(song_id);
        Some(
            slides
                .into_iter()
                .map(|slide| {
                    let slide_content = SlideContent::for_slide(&state, &slide);
                    view! {
                        <div class="preview">
                            <Slide slide=slide_content />
                        </div>
                    }
                })
                .collect::<Vec<_>>(),
        )
    }
}

fn save_song(
    event: SubmitEvent,
    write_state: WriteSignal<State>,
    song_id: u32,
    title: HtmlInputElement,
    authors: HtmlInputElement,
    songbook_entries: HtmlInputElement,
    verseorder: HtmlInputElement,
    lyrics: HtmlTextAreaElement,
) {
    event.prevent_default();
    let title = title.value().trim().to_string();
    let authors = authors.value();
    let songbook_entries = songbook_entries.value();
    let verseorder = verseorder.value().trim().to_string();
    let lyrics = lyrics.value();

    write_state.update(|state| {
        let Some(song) = state.songs.get_mut(&song_id) else {
            return;
        };

        song.properties.titles.titles[0].title = title;
        set_authors_from_string(song, &authors);
        set_songbooks_from_string(song, &songbook_entries);
        song.properties.verse_order = if verseorder.is_empty() {
            None
        } else {
            Some(verseorder)
        };
        set_lyrics_from_text(song, &lyrics);
    });
}
