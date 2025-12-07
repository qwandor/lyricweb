// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod files;
mod import_export;
mod model;
mod playlist;
mod slide;
mod songlist;

use crate::{
    import_export::{export, import, import_url},
    model::{PlaylistEntry, SlideIndex, State},
    playlist::Playlist,
    slide::CurrentSlide,
    songlist::SongList,
};
use leptos::{
    prelude::*,
    server::codee::string::{FromToStringCodec, JsonSerdeCodec, OptionCodec},
    task::spawn_local,
};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::{query_signal, use_navigate},
    path,
};
use leptos_use::storage::use_local_storage;
use std::cell::RefCell;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, PresentationRequest, SubmitEvent, Window};

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
                <Route path=path!("*any") view={move || if query_signal("present").0.get().unwrap_or_default() {
                    view! {
                        <CurrentSlide state current_slide/>
                    }.into_any()
                } else if let Some(url) = query_signal::<String>("import_url").0.get() {
                    view! {
                        <ImportUrl url write_state />
                    }.into_any()
                } else {
                    view! {
                        <Controller state write_state current_slide write_current_slide/>
                    }.into_any()
                }
            }/>
            </Routes>
        </Router>
    }
}

#[component]
fn ImportUrl(url: String, write_state: WriteSignal<State>) -> impl IntoView {
    let (error, write_error) = signal(None);

    view! {
        <p>"Import '" {url.clone()} "'?"</p>
        <form on:submit=move |event| { let url = url.clone(); spawn_show_error(import_url(event, url, write_state, use_navigate()), write_error); }>
            <input type="submit" value="Import" />
        </form>
        <p id="error">{ error }</p>
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
                <div class="button-row">
                    <form on:submit=move |event| spawn_show_error(import(event, write_state, write_output), write_error) >
                        <input type="submit" value="Import" />
                    </form>
                    <form on:submit=move |event| spawn_show_error(export(event, state), write_error) >
                        <input type="submit" value="Export" />
                    </form>
                </div>
                <div>
                    <p id="output">{ output }</p>
                    <p id="error">{ error }</p>
                </div>
                <SongList state write_state current_playlist />
                <div class="button-row">
                    <form class="wide" on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap(), current_playlist, write_state)>
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
                    <input type="button" value="Present in window" on:click=move |_| open_presentation(&mut presentation_window.borrow_mut())/>
                    <input type="button" value="Present on external screen" on:click=move |_| spawn_local(open_external_presentation())/>
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

/// Opens the presentation on an external monitor.
async fn open_external_presentation() {
    let request = PresentationRequest::new_with_url("?present=true").unwrap();
    JsFuture::from(request.start().unwrap()).await.unwrap();
}

fn show_error(result: Result<(), String>, write_error: WriteSignal<Option<String>>) {
    write_error.set(result.err());
}

fn spawn_show_error(
    fut: impl Future<Output = Result<(), String>> + 'static,
    write_error: WriteSignal<Option<String>>,
) {
    spawn_local((async move || show_error(fut.await, write_error))())
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
