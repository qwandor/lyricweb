// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Utilities for working with files.

use gloo_file::File;
use leptos::tachys::dom::window;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FilePickerAcceptType, FileSystemFileHandle, FileSystemWritableFileStream,
    OpenFilePickerOptions, SaveFilePickerOptions,
    js_sys::{Array, Object, Reflect},
};

pub fn make_accept_type(description: &str, accept: &[(&str, &[&str])]) -> FilePickerAcceptType {
    let accept_type = FilePickerAcceptType::new();
    accept_type.set_description(description);
    let accept_object = Object::new_typed();
    for (mimetype, extensions) in accept {
        Reflect::set(
            &accept_object,
            &JsValue::from_str(mimetype),
            &extensions
                .into_iter()
                .map(|extension| JsValue::from_str(extension))
                .collect::<Array>(),
        )
        .unwrap();
    }
    accept_type.set_accept(&accept_object);
    accept_type
}

/// Prompts the user to pick a file to open.
pub async fn pick_open_file(options: &OpenFilePickerOptions) -> Result<File, JsValue> {
    let file_handles = JsFuture::from(window().show_open_file_picker_with_options(options)?)
        .await?
        .unchecked_into::<Array>()
        .iter()
        .map(JsValue::unchecked_into::<FileSystemFileHandle>)
        .collect::<Vec<_>>();
    Ok(JsFuture::from(file_handles.first().unwrap().get_file())
        .await?
        .unchecked_into::<web_sys::File>()
        .into())
}

/// Prompts the user to pick a file to save to.
pub async fn pick_save_file(
    options: &SaveFilePickerOptions,
) -> Result<FileSystemWritableFileStream, JsValue> {
    Ok(JsFuture::from(
        JsFuture::from(window().show_save_file_picker_with_options(options)?)
            .await?
            .unchecked_into::<FileSystemFileHandle>()
            .create_writable(),
    )
    .await?
    .unchecked_into::<FileSystemWritableFileStream>())
}

/// Writes the given contents to the given file and then closes it.
pub async fn write_and_close(
    file: &FileSystemWritableFileStream,
    contents: &str,
) -> Result<(), JsValue> {
    JsFuture::from(file.write_with_str(contents)?).await?;
    JsFuture::from(file.close()).await?;
    Ok(())
}
