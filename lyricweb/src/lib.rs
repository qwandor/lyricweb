use gloo_file::{File, FileList, futures::read_as_text};
use gloo_utils::document;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, Event, EventTarget, HtmlInputElement};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    add_async_listener_by_id("file", "change", file_changed);
}

fn get_element_by_id(id: &str) -> Element {
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Failed to find element {id}"))
}

fn add_async_listener_by_id<F: Future<Output = ()> + 'static>(
    id: &str,
    event_type: &str,
    callback: fn(Event) -> F,
) {
    add_listener_by_id(id, event_type, move |event| spawn_local(callback(event)));
}

fn add_listener_by_id(id: &str, event_type: &str, callback: impl Fn(Event) + 'static) {
    add_listener_and_leak(&get_element_by_id(id), event_type, callback);
}

fn add_listener_and_leak(
    target: &EventTarget,
    event_type: &str,
    callback: impl Fn(Event) + 'static,
) {
    target
        .add_event_listener_with_callback(
            event_type,
            Closure::<dyn Fn(Event)>::new(callback)
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap();
}

async fn file_changed(_event: Event) {
    let files = FileList::from(
        get_element_by_id("file")
            .unchecked_into::<HtmlInputElement>()
            .files()
            .unwrap(),
    );
    open_file(files.first().unwrap()).await;
}

async fn open_file(file: &File) {
    show_output(&format!(
        "{}: {} bytes, {}",
        file.name(),
        file.size(),
        file.raw_mime_type()
    ));
    let text = read_as_text(&file).await.unwrap();
    show_output(&text);
}

fn show_output(text: &str) {
    let error_element = document()
        .get_element_by_id("output")
        .expect("Couldn't find output element");
    error_element.set_text_content(Some(text));
}

fn show_error(error: &str) {
    let error_element = document()
        .get_element_by_id("error")
        .expect("Couldn't find error element");
    error_element.set_text_content(Some(error));
}
