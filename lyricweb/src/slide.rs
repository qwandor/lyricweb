// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{Slide, SlideIndex, State, title_for_song};
use leptos::prelude::*;
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};

#[component]
pub fn CurrentSlide(
    state: Signal<State>,
    current_slide: Signal<Option<SlideIndex>>,
) -> impl IntoView {
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
