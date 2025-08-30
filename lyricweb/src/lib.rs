use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, EventTarget, File, HtmlFormElement, HtmlInputElement};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    add_listener_by_id("file", "change", file_changed);
}

fn get_element_by_id(id: &str) -> Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Failed to find element {id}"))
}

fn add_listener_by_id(id: &str, event_type: &str, callback: fn(Event)) {
    add_listener_and_leak(&get_element_by_id(id), event_type, callback);
}

fn add_listener_and_leak(target: &EventTarget, event_type: &str, callback: fn(Event)) {
    target
        .add_event_listener_with_callback(
            event_type,
            Closure::<dyn Fn(Event)>::new(callback)
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap();
}

fn file_changed(_event: Event) {
    open_file(
        &get_element_by_id("file")
            .unchecked_into::<HtmlInputElement>()
            .files()
            .unwrap()
            .get(0)
            .unwrap(),
    );
}

fn open_file(file: &File) {
    show_output(&file.name());
}

fn show_output(text: &str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let error_element = document
        .get_element_by_id("output")
        .expect("Couldn't find output element");
    error_element.set_text_content(Some(text));
}

fn show_error(error: &str) {
    let document = web_sys::window()
        .expect("Couldn't find window")
        .document()
        .expect("Couldn't find document");
    let error_element = document
        .get_element_by_id("error")
        .expect("Couldn't find error element");
    error_element.set_text_content(Some(error));
}
