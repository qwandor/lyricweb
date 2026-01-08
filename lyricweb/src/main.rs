// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod editsong;
mod files;
mod import_export;
mod model;
mod playlist;
mod slide;
mod songlist;

use crate::{
    editsong::{EditSong, PreviewSlides},
    import_export::{export, import, import_url},
    model::{PlaylistEntry, SlideIndex, State, slide::SlideContent},
    playlist::Playlist,
    slide::{PresentationReceiver, Slide},
    songlist::SongList,
};
use leptos::{
    ev::{Custom, change, message},
    prelude::*,
    server::codee::string::{FromToStringCodec, JsonSerdeCodec, OptionCodec},
    task::spawn_local,
};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::{query_signal, use_navigate},
    path,
};
use leptos_use::{storage::use_local_storage, use_event_listener};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Event, HtmlTextAreaElement, KeyboardEvent, PresentationAvailability, PresentationConnection,
    PresentationConnectionAvailableEvent, PresentationConnectionState, PresentationRequest,
    SubmitEvent, Window,
};

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
    let current_slide_content = move || {
        if let Some(current_slide) = current_slide.get() {
            SlideContent::for_index(&state.read(), current_slide).unwrap_or_default()
        } else {
            Default::default()
        }
    };

    view! {
        <Router>
            <Routes fallback=|| "Not found">
                <Route path=path!("*any") view={move || if query_signal("present").0.get().unwrap_or_default() {
                    view! {
                        <div id="presentation" tabindex="0" on:keydown=move |event| presentation_keydown(event, state, write_current_slide)>
                            <Slide slide=current_slide_content/>
                        </div>
                    }.into_any()
                } else if query_signal("present_remote").0.get().unwrap_or_default() {
                    view! {
                        <PresentationReceiver />
                    }.into_any()
                } else if let Some(url) = query_signal::<String>("import_url").0.get() {
                    view! {
                        <ImportUrl url write_state />
                    }.into_any()
                } else {
                    view! {
                        <Controller state write_state current_slide write_current_slide current_slide_content/>
                    }.into_any()
                }
            }/>
            </Routes>
        </Router>
    }
}

fn presentation_keydown(
    event: KeyboardEvent,
    state: Signal<State>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
) {
    match event.key().as_str() {
        "ArrowLeft" => {
            event.prevent_default();
            previous_slide(write_current_slide, state);
        }
        "ArrowRight" => {
            event.prevent_default();
            next_slide(write_current_slide, state);
        }
        _ => {}
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
    #[prop(into)] current_slide_content: Signal<SlideContent>,
) -> impl IntoView {
    let text_entry = NodeRef::new();

    let (current_playlist, write_current_playlist, _) =
        use_local_storage::<_, OptionCodec<FromToStringCodec>>("current_playlist");
    let no_current_playlist = move || current_playlist.get().is_none();
    let current_slide_text = move || {
        let index = current_slide.get()?;
        state.read().slide_text(index)
    };
    let no_previous_slide = move || {
        if let Some(current_slide) = current_slide.get() {
            !current_slide.has_previous()
        } else {
            false
        }
    };
    let no_next_slide = move || {
        if let Some(current_slide) = current_slide.get() {
            !current_slide.has_next(&state.read())
        } else {
            false
        }
    };

    if current_playlist.get_untracked().is_none()
        && let Some((&playlist_id, _)) = state.get_untracked().playlists.first_key_value()
    {
        write_current_playlist.set(Some(playlist_id));
    }

    let (output, write_output) = signal(None);
    let (error, write_error) = signal(None);

    let (edit_song, write_edit_song) = signal(None);

    let presentation_window = StoredValue::new_local(None);

    let (presentation_displays_available, presentation_connection, presentation_request) =
        presentation_init(
            current_slide_content,
            write_error,
            write_current_slide,
            state,
        );

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
                <SongList state write_state current_playlist write_edit_song />
            </div>
            <div class="column" id="playlist-column">
                <Show when=move || edit_song.get().is_some()
                fallback=move || view! {
                    <Playlist state write_state current_playlist write_current_playlist current_slide write_current_slide/>
                    <form class="vertical" on:submit=move |event| add_text_to_playlist(event, text_entry.get().unwrap(), current_playlist, write_current_slide, write_state)>
                        <textarea rows="6" node_ref=text_entry prop:value=move || current_slide_text().unwrap_or_default() />
                        <div class="button-row">
                            <input type="submit" value="Add" disabled=no_current_playlist />
                            <input type="button" value="Update"
                                disabled=move || current_slide_text().is_none()
                                on:click=move |_| update_text_in_playlist(text_entry.get().unwrap(), current_slide, write_state) />
                        </div>
                    </form>
                }>
                    <EditSong state write_state edit_song write_edit_song/>
                </Show>
            </div>
            <div class="column">
                <Show when=move || edit_song.get().is_some()
                fallback=move || view! {
                    <form>
                        <input type="button" value="Present in window" on:click=move |_| open_presentation(&mut presentation_window.write_value())/>
                        {move || {
                            if presentation_connection.read().is_some() {
                                view! {
                                    <input type="button" value="Stop presenting" on:click=move |_| show_error(close_external_presentation(presentation_connection), write_error)/>
                                }.into_any()
                            } else if presentation_displays_available.get() {
                                view! {
                                    <input type="button" value="Present on external screen" on:click=move |_| {
                                        spawn_show_error(open_external_presentation(presentation_request.read_value().clone().unwrap()), write_error)
                                    } />
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }
                        }}
                    </form>
                    <div class="preview">
                        <Slide slide=current_slide_content/>
                    </div>
                    <form>
                        <input type="button" value="Previous" disabled=no_previous_slide on:click=move |_| previous_slide(write_current_slide, state) />
                        <input type="button" value="Next" disabled=no_next_slide on:click=move |_| next_slide(write_current_slide, state) />
                    </form>
                    <ThemeSettings state write_state />
                }>
                    <PreviewSlides state song_id=edit_song />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn ThemeSettings(state: Signal<State>, write_state: WriteSignal<State>) -> impl IntoView {
    view! {
        <form>
            <h2>Theme</h2>
            <table>
                <tr>
                    <td>Heading size</td>
                    <td><input type="number" min="10" max="100" step="5"
                        prop:value=move || state.read().theme.heading_size
                        on:change:target=move |event| if let Ok(heading_size) = event.target().value().parse() {
                            write_state.write().theme.heading_size = heading_size;
                        }
                    /></td>
                </tr>
                <tr>
                    <td>Heading colour</td>
                    <td><input type="color"
                        prop:value=move || state.read().theme.heading_colour.clone()
                        on:change:target=move |event| write_state.write().theme.heading_colour = event.target().value()
                    /></td>
                </tr>
                <tr>
                    <td>Body size</td>
                    <td><input type="number" min="10" max="100" step="5"
                        prop:value=move || state.read().theme.body_size
                        on:change:target=move |event| if let Ok(size) = event.target().value().parse() {
                            write_state.write().theme.body_size = size;
                        }
                    /></td>
                </tr>
                <tr>
                    <td>Body colour</td>
                    <td><input type="color"
                        prop:value=move || state.read().theme.body_colour.clone()
                        on:change:target=move |event| write_state.write().theme.body_colour = event.target().value()
                    /></td>
                </tr>
                <tr>
                    <td>Background colour</td>
                    <td><input type="color"
                        prop:value=move || state.read().theme.background_colour.clone()
                        on:change:target=move |event| write_state.write().theme.background_colour = event.target().value()
                    /></td>
                </tr>
                <tr>
                    <td>Font</td>
                    <td><input type="text"
                        prop:value=move || state.read().theme.font_family.clone()
                        on:change:target=move |event| write_state.write().theme.font_family = event.target().value()
                    /></td>
                </tr>
            </table>
        </form>
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

fn presentation_init(
    current_slide_content: Signal<SlideContent>,
    write_error: WriteSignal<Option<String>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
    state: Signal<State>,
) -> (
    ReadSignal<bool>,
    RwSignal<Option<PresentationConnection>, LocalStorage>,
    StoredValue<Option<PresentationRequest>, LocalStorage>,
) {
    let (presentation_displays_available, write_presentation_displays_available) = signal(false);
    let presentation_connection = RwSignal::new_local(None);

    let presentation_request = PresentationRequest::new_with_url("?present_remote=true").ok();
    if let Some(presentation_request) = &presentation_request {
        show_error(
            setup_presentation_request(
                &presentation_request,
                current_slide_content,
                presentation_connection,
                write_current_slide,
                state,
            ),
            write_error,
        );
        spawn_show_error(
            listen_presentation_availability(
                presentation_request.clone(),
                write_presentation_displays_available,
            ),
            write_error,
        );
    }
    let presentation_request = StoredValue::new_local(presentation_request);

    (
        presentation_displays_available,
        presentation_connection,
        presentation_request,
    )
}

fn setup_presentation_request(
    request: &PresentationRequest,
    current_slide_content: Signal<SlideContent>,
    presentation_connection: RwSignal<Option<PresentationConnection>, LocalStorage>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
    state: Signal<State>,
) -> Result<(), String> {
    Effect::new(move || {
        let data = serde_json::to_string(&*current_slide_content.read()).unwrap();
        if let Some(connection) = presentation_connection.read().as_ref() {
            if connection.state() == PresentationConnectionState::Connected {
                gloo_console::log!(format!("Sending {data}"));
                connection.send_with_str(&data).unwrap();
            }
        }
    });

    if let Ok(Some(presentation)) = window().navigator().presentation() {
        presentation.set_default_request(Some(request));
    }

    _ = use_event_listener(
        request.clone(),
        Custom::new("connectionavailable"),
        move |event: PresentationConnectionAvailableEvent| {
            gloo_console::log!(&event);

            let connection = event.connection();
            presentation_connection.write().replace(connection.clone());

            _ = use_event_listener(
                connection.clone(),
                Custom::new("terminate"),
                move |event: Event| {
                    gloo_console::log!(&event);
                    presentation_connection.set(None);
                },
            );

            let connection_clone = connection.clone();
            _ = use_event_listener(
                connection.clone(),
                Custom::new("connect"),
                move |event: Event| {
                    gloo_console::log!(&event);
                    let data =
                        serde_json::to_string(&*current_slide_content.read_untracked()).unwrap();
                    gloo_console::log!(format!("Connect event, sending {data}"));
                    connection_clone.send_with_str(&data).unwrap();
                },
            );

            let connection_clone = connection.clone();
            _ = use_event_listener(connection.clone(), message, move |event| {
                if let Some(message_data) = event.data().as_string() {
                    match message_data.as_str() {
                        "prev" => {
                            previous_slide(write_current_slide, state);
                        }
                        "next" => {
                            next_slide(write_current_slide, state);
                        }
                        _ => {
                            let data =
                                serde_json::to_string(&*current_slide_content.read_untracked())
                                    .unwrap();
                            connection_clone.send_with_str(&data).unwrap();
                        }
                    }
                }
            });

            if connection.state() == PresentationConnectionState::Connected {
                let data = serde_json::to_string(&*current_slide_content.read_untracked()).unwrap();
                gloo_console::log!(format!("Connected already, sending {data}"));
                connection.send_with_str(&data).unwrap();
            }
        },
    );

    Ok(())
}

async fn listen_presentation_availability(
    request: PresentationRequest,
    write_presentation_displays_available: WriteSignal<bool>,
) -> Result<(), String> {
    let availability = JsFuture::from(request.get_availability().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?
        .unchecked_into::<PresentationAvailability>();

    write_presentation_displays_available.set(availability.value());
    _ = use_event_listener(availability.clone(), change, move |_| {
        write_presentation_displays_available.set(availability.value());
    });

    Ok(())
}

fn close_external_presentation(
    presentation_connection: RwSignal<Option<PresentationConnection>, LocalStorage>,
) -> Result<(), String> {
    let connection = presentation_connection.get();
    if let Some(connection) = connection {
        connection.terminate().map_err(|e| format!("{e:?}"))?;
    }
    Ok(())
}

/// Opens the presentation on an external monitor.
async fn open_external_presentation(request: PresentationRequest) -> Result<(), String> {
    let connection = JsFuture::from(request.start().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?
        .unchecked_into::<PresentationConnection>();

    gloo_console::log!(&connection);

    Ok(())
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
    text_entry: HtmlTextAreaElement,
    current_playlist: Signal<Option<u32>>,
    write_current_slide: WriteSignal<Option<SlideIndex>>,
    write_state: WriteSignal<State>,
) {
    event.prevent_default();

    let Some(current_playlist) = current_playlist.get() else {
        return;
    };

    let text = text_entry.value();
    write_state.update(|state| {
        let entries = &mut state.playlists.get_mut(&current_playlist).unwrap().entries;
        entries.push(PlaylistEntry::Text(text));
        write_current_slide.set(Some(SlideIndex {
            playlist_id: current_playlist,
            entry_index: entries.len() - 1,
            page_index: 0,
        }));
    });
    text_entry.set_value("");
}

fn update_text_in_playlist(
    text_entry: HtmlTextAreaElement,
    current_slide: Signal<Option<SlideIndex>>,
    write_state: WriteSignal<State>,
) {
    let Some(current_slide) = current_slide.get() else {
        return;
    };

    let new_text = text_entry.value();

    write_state.update(|state| {
        if let PlaylistEntry::Text(text) = &mut state
            .playlists
            .get_mut(&current_slide.playlist_id)
            .unwrap()
            .entries[current_slide.entry_index]
        {
            *text = new_text;
        }
    });
}

fn previous_slide(write_current_slide: WriteSignal<Option<SlideIndex>>, state: Signal<State>) {
    write_current_slide.update(|current_slide| {
        if let Some(current_slide) = current_slide {
            current_slide.previous(&state.read());
        }
    });
}

fn next_slide(write_current_slide: WriteSignal<Option<SlideIndex>>, state: Signal<State>) {
    write_current_slide.update(|current_slide| {
        if let Some(current_slide) = current_slide {
            current_slide.next(&state.read());
        }
    });
}
