// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Types deriving the appropriate serde traits to be used with
//! [`quick-xml`](https://crates.io/crates/quick-xml) for parsing
//! [OpenLyrics](https://docs.openlyrics.org/en/latest/) XML files, and some helper functions and
//! methods for common tasks.
//!
//! [`types::Song`] is the top-level type for parsing an OpenLyrics file, e.g.:
//!
//! ```no_run
//! use openlyrics::types::Song;
//! use quick_xml::de::from_reader;
//! use std::{fs::File, io::BufReader};
//!
//! let song = from_reader::<_, Song>(BufReader::new(File::open("song.xml")?))?;
//! println!("Title: {}", song.properties.titles.titles[0].title);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod types;

use crate::types::VerseContent;

/// Converts the contents of a verse to a vector of lines, ignoring chords, tags and comments.
pub fn simplify_contents(contents: &[VerseContent]) -> Vec<String> {
    let mut simple_lines = vec![];
    add_simple_contents(contents, &mut simple_lines);
    simple_lines
        .into_iter()
        .map(|line| line.trim().to_owned())
        .collect()
}

fn add_simple_contents(contents: &[VerseContent], simple_lines: &mut Vec<String>) {
    for content in contents {
        add_simple_content(content, simple_lines);
    }
}

fn add_simple_content(content: &VerseContent, simple_lines: &mut Vec<String>) {
    match content {
        VerseContent::Text(text) => {
            if simple_lines.is_empty() {
                simple_lines.push(String::new());
            }
            let text = text.replace(char::is_whitespace, " ");
            let line = simple_lines.last_mut().unwrap();
            line.push_str(text.trim());
            if text.ends_with(' ') {
                line.push(' ');
            }
        }
        VerseContent::Chord { contents, .. } => {
            add_simple_contents(contents, simple_lines);
        }
        VerseContent::Br => {
            simple_lines.push(String::new());
        }
        VerseContent::Comment(_) => {}
        VerseContent::Tag { contents, .. } => {
            add_simple_contents(contents, simple_lines);
        }
    }
}
