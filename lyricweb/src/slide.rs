// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::slide::SlideContent;
use leptos::prelude::*;

#[component]
pub fn Slide(#[prop(into)] slide: Signal<SlideContent>) -> impl IntoView {
    move || {
        let content = slide.read();
        view! {
            <div class="slide">
            { content.title.as_ref().map(|title| {
                view! {
                    <h1>{title.clone()}</h1>
                }})
            }
            <p>
                { content.lines.iter().map(|line| {
                    let text = line.text.clone();
                    match (line.bold, line.italic) {
                        (false, false) => view! { {text}<br/> }.into_any(),
                        (true, false) => view! { <strong>{text}</strong><br/> }.into_any(),
                        (false, true) => view! { <em>{text}</em><br/> }.into_any(),
                        (true, true) => view! { <strong><em>{text}</em></strong><br/> }.into_any(),
                    }
                } ).collect::<Vec<_>>() }
            </p>
            </div>
        }
    }
}
