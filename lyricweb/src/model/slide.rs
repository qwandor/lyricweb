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
    pub lines: Vec<SlideLine>,
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
            Slide::Lyrics {
                song_id,
                lyric_entry_index,
                lines_index,
            } => {
                let song = &state.songs[song_id];
                Some(Self::song_page(
                    song,
                    *lyric_entry_index,
                    *lines_index,
                    theme,
                ))
            }
            Slide::Text(text) => Some(Self {
                title: None,
                lines: vec![SlideLine {
                    text: (*text).to_owned(),
                    ..Default::default()
                }],
                credit: None,
                theme,
            }),
        }
    }

    fn song_page(song: &Song, lyric_entry_index: usize, lines_index: usize, theme: Theme) -> Self {
        let item = &song.lyrics.lyrics[lyric_entry_index];

        let title = if lyric_entry_index == 0 && lines_index == 0 {
            Some(title_for_song(song).to_owned())
        } else {
            None
        };

        let credit = if lyric_entry_index == song.lyrics.lyrics.len() - 1
            && match item {
                LyricEntry::Verse { lines, .. } => lines_index == lines.len() - 1,
                LyricEntry::Instrument { .. } => true,
            } {
            Some(authors_as_string(song))
        } else {
            None
        };

        let lines =
            match item {
                LyricEntry::Verse { name, lines, .. } => {
                    let line = &lines[lines_index];
                    let mut lines = Vec::new();

                    if let Some(part) = line.part.as_ref() {
                        lines.push(SlideLine {
                            text: format!("({part})"),
                            bold: true,
                            ..Default::default()
                        });
                    }

                    let mut before_first_line = if name.starts_with('v') && lines_index == 0 {
                        Some(format!("{}. ", &name[1..]))
                    } else {
                        None
                    };

                    lines.extend(simplify_contents(&line.contents).into_iter().map(|line| {
                        SlideLine {
                            text: before_first_line.take().unwrap_or_default() + &line,
                            ..Default::default()
                        }
                    }));

                    if let Some(repeat) = line.repeat {
                        lines.push(SlideLine {
                            text: format!("x {repeat}"),
                            bold: true,
                            ..Default::default()
                        });
                    }
                    lines
                }
                LyricEntry::Instrument { name, .. } => {
                    vec![SlideLine {
                        text: format!("(instrumental {name})"),
                        ..Default::default()
                    }]
                }
            };

        Self {
            title,
            lines,
            credit,
            theme,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SlideLine {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
}
