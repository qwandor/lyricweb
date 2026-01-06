// Copyright 2026 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

mod abc;
mod music_xml;

pub use crate::{abc::tunebook_to_open_lyrics, music_xml::music_xml_to_open_lyrics};
use openlyrics::types::{Lines, VerseContent};

fn lines_to_open_lyrics(verse_lyrics: Vec<String>) -> Lines {
    let mut contents = Vec::new();
    for line in verse_lyrics {
        if line.is_empty() {
            continue;
        }
        if !contents.is_empty() {
            contents.push(VerseContent::Br);
        }
        contents.push(VerseContent::Text(line));
    }
    Lines {
        contents,
        ..Default::default()
    }
}
