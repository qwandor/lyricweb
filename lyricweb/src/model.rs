// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use openlyrics::types::{LyricEntry, Song};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct State {
    pub songs: Vec<Song>,
    pub playlist: Vec<PlaylistEntry>,
    pub current_slide: usize,
}

impl State {
    pub const fn new() -> Self {
        Self {
            songs: Vec::new(),
            playlist: Vec::new(),
            current_slide: 0,
        }
    }

    pub fn slides(&self) -> Vec<Slide> {
        let mut slides = Vec::new();
        for entry in &self.playlist {
            match entry {
                PlaylistEntry::Song { song_index } => {
                    let song = &self.songs[*song_index];
                    for (lyric_entry_index, item) in song.lyrics.lyrics.iter().enumerate() {
                        match item {
                            LyricEntry::Verse { lines, .. } => {
                                for lines_index in 0..lines.len() {
                                    slides.push(Slide::Lyrics {
                                        song_index: *song_index,
                                        lyric_entry_index,
                                        lines_index,
                                    })
                                }
                            }
                            LyricEntry::Instrument { .. } => slides.push(Slide::Lyrics {
                                song_index: *song_index,
                                lyric_entry_index,
                                lines_index: 0,
                            }),
                        }
                    }
                }
                PlaylistEntry::Text(text) => slides.push(Slide::Text(&text)),
            }
        }
        slides
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Slide<'a> {
    Lyrics {
        song_index: usize,
        lyric_entry_index: usize,
        lines_index: usize,
    },
    Text(&'a str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaylistEntry {
    Song { song_index: usize },
    Text(String),
}

impl PlaylistEntry {
    pub fn summary<'a>(&'a self, state: &'a State) -> &'a str {
        match self {
            PlaylistEntry::Song { song_index } => {
                &state.songs[*song_index].properties.titles.titles[0].title
            }
            PlaylistEntry::Text(text) => text,
        }
    }

    /// Returns a list of verse titles for songs, or `None` for text.
    pub fn pages<'a>(&self, state: &'a State) -> Option<Vec<&'a str>> {
        match self {
            PlaylistEntry::Song { song_index } => Some(
                state.songs[*song_index]
                    .lyrics
                    .lyrics
                    .iter()
                    .map(|lyric_entry| lyric_entry.name())
                    .collect(),
            ),
            PlaylistEntry::Text(_) => None,
        }
    }
}
