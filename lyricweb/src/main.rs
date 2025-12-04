// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod model;
mod playlist;
mod slide;
mod songlist;

use crate::{
    model::{PlaylistEntry, SlideIndex, State},
    playlist::Playlist,
    slide::CurrentSlide,
    songlist::SongList,
};
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
use quick_xml::de::from_str;
use std::cell::RefCell;
use web_sys::{Event, HtmlInputElement, SubmitEvent, Window};

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

    if current_playlist.get_untracked().is_none()
        && let Some((&playlist_id, _)) = state.get_untracked().playlists.first_key_value()
    {
        write_current_playlist.set(Some(playlist_id));
    }

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
        <div class="button-row">
        <form on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap(), current_playlist, write_state)>
        <input type="text" node_ref=text_entry />
        <input type="submit" value="Add to playlist" disabled=no_current_playlist />
        </form>
        </div>
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
    text_entry.set_value("");
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
