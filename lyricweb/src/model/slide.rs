// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{
    Slide, SlideIndex, State, Theme,
    helpers::{authors_as_string, title_for_song},
};
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};
use serde::{Deserialize, Serialize};

/// The contents of a slide ready to render.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SlideContent {
    pub title: Option<String>,
    pub body: Option<String>,
    pub credit: Option<String>,
    pub theme: Theme,
}

impl SlideContent {
    pub fn for_index(state: &State, index: SlideIndex) -> Option<Self> {
        let slide = &state.slide(index)?;
        Self::for_slide(state, slide)
    }

    pub fn for_slide(state: &State, slide: &Slide) -> Option<Self> {
        let theme = state.theme.clone();
        match slide {
            Slide::SongStart { .. } => None,
            &Slide::Lyrics {
                song_id,
                lyric_entry_index,
                lines_index,
                last_page,
            } => {
                let song = &state.songs[&song_id];
                Some(Self::song_page(
                    song,
                    lyric_entry_index,
                    lines_index,
                    last_page,
                    theme,
                ))
            }
            Slide::Text(text) => Some(Self::for_text(text, theme)),
        }
    }

    fn for_text(text: &str, theme: Theme) -> Self {
        Self {
            title: None,
            body: Some(text.to_owned()),
            credit: None,
            theme,
        }
    }

    fn song_page(
        song: &Song,
        lyric_entry_index: usize,
        lines_index: usize,
        last_page: bool,
        theme: Theme,
    ) -> Self {
        let item = &song.lyrics.lyrics[lyric_entry_index];

        let title = if lyric_entry_index == 0 && lines_index == 0 {
            Some(title_for_song(song).to_owned())
        } else {
            None
        };

        let credit = if last_page {
            Some(authors_as_string(song))
        } else {
            None
        };

        let body = match item {
            LyricEntry::Verse { name, lines, .. } => {
                let line = &lines[lines_index];
                let mut body = String::new();

                if let Some(part) = line.part.as_ref() {
                    body += &format!("<strong>({part})</strong><br/>");
                }

                if name.starts_with('v') && lines_index == 0 {
                    body += &format!("{}. ", &name[1..]);
                }

                for line in simplify_contents(&line.contents) {
                    body += &line;
                    body += "<br/>";
                }

                if let Some(repeat) = line.repeat {
                    body += &format!("<em>x {repeat}</em><br/>");
                }
                body
            }
            LyricEntry::Instrument { name, .. } => {
                format!("(instrumental {name})")
            }
        };

        Self {
            title,
            body: Some(body),
            credit,
            theme,
        }
    }
}
