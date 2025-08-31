// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use openlyrics::types::Song;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaylistEntry {
    Song { song: usize },
    Text(String),
}

impl PlaylistEntry {
    pub fn summary<'a>(&'a self, state: &'a State) -> &'a str {
        match self {
            PlaylistEntry::Song { song } => &state.songs[*song].properties.titles.titles[0].title,
            PlaylistEntry::Text(text) => text,
        }
    }
}
