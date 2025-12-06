// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Utilities for working with files.

use gloo_file::File;
use leptos::tachys::dom::window;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemFileHandle, FileSystemWritableFileStream, OpenFilePickerOptions,
    SaveFilePickerOptions, js_sys::Array,
};

/// Type for the `types` entries of [`OpenFilePickerOptions`] and [`SaveFilePickerOptions`].
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
pub struct FileType {
    pub description: Option<String>,
    pub accept: BTreeMap<String, Vec<String>>,
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
