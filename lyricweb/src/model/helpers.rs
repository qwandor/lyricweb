// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use openlyrics::{
    simplify_contents,
    types::{Author, LyricEntry, Song},
};
use std::fmt::Write;

/// Returns the title to use for the given song.
pub fn title_for_song(song: &Song) -> &str {
    &song.properties.titles.titles[0].title
}

/// Returns whether the given song should be displayed when the given search filter is entered.
pub fn song_matches_filter(song: &Song, filter: &str) -> bool {
    title_for_song(song)
        .to_lowercase()
        .contains(&filter.to_lowercase())
}

/// Returns the first line of the given lyric entry and lines of the given song, if any.
pub fn first_line(song: &Song, lyric_entry_index: usize, lines_index: usize) -> Option<String> {
    let lyric_entry = song.lyrics.lyrics.get(lyric_entry_index)?;
    if let LyricEntry::Verse { lines, .. } = lyric_entry {
        simplify_contents(&lines.get(lines_index)?.contents)
            .into_iter()
            .next()
    } else {
        None
    }
}

/// Returns the full lyrics of the given song as a single string, for editing.
pub fn lyrics_as_text(song: &Song) -> String {
    let mut text = String::new();
    for lyric_entry in &song.lyrics.lyrics {
        if let LyricEntry::Verse { name, lines, .. } = lyric_entry {
            if !text.is_empty() {
                writeln!(&mut text).unwrap();
            }
            writeln!(&mut text, "{name}:").unwrap();
            for lines_entry in lines {
                for line in simplify_contents(&lines_entry.contents) {
                    writeln!(&mut text, "{line}").unwrap();
                }
            }
        }
    }
    text
}

/// Returns the authors of the given song as a single string, for displaying or editing.
pub fn authors_as_string(song: &Song) -> String {
    song.properties
        .authors
        .authors
        .iter()
        .map(|author| {
            let author_name = &author.name;
            if let Some(author_type) = &author.author_type {
                format!("{author_name} ({author_type})")
            } else {
                author_name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Sets the authors of the given song from a string of the format returned by `authors_as_string`.
pub fn set_authors_from_string(song: &mut Song, authors: &str) {
    song.properties.authors.authors = authors
        .split(',')
        .map(|author| {
            let author = author.trim();
            if let Some((name, rest)) = author.split_once('(')
                && rest.ends_with(')')
            {
                Author {
                    author_type: Some(rest.trim_end_matches(')').trim().to_string()),
                    lang: None,
                    name: name.trim().to_string(),
                }
            } else {
                Author {
                    author_type: None,
                    lang: None,
                    name: author.to_string(),
                }
            }
        })
        .collect();
}
