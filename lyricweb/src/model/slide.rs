// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::model::{
    Slide, SlideIndex, State, Theme,
    helpers::{authors_as_string, first_songbook, title_for_song},
};
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Song},
};
use pulldown_cmark::{Parser, html::push_html};
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
        Some(Self::for_slide(state, slide))
    }

    pub fn for_slide(state: &State, slide: &Slide) -> Self {
        let theme = state.theme.clone();
        match slide {
            Slide::SongStart { song_id, .. } => {
                let song = &state.songs[&song_id];
                Self::song_title(song, theme)
            }
            &Slide::Lyrics {
                song_id,
                lyric_entry_index,
                lines_index,
                last_page,
            } => {
                let song = &state.songs[&song_id];
                Self::song_page(song, lyric_entry_index, lines_index, last_page, theme)
            }
            Slide::Text(text) => Self::for_text(text, theme),
        }
    }

    fn for_text(text: &str, theme: Theme) -> Self {
        let parser = Parser::new(text);
        let mut body = String::new();
        push_html(&mut body, parser);
        Self {
            title: None,
            body: Some(body),
            credit: None,
            theme,
        }
    }

    fn song_title(song: &Song, theme: Theme) -> Self {
        let title = Some(if let Some(songbook_entry) = first_songbook(song) {
            format!("Hymn {songbook_entry}")
        } else {
            format!("Hymn")
        });
        let body = Some(format!("<h2>{}</h2>", title_for_song(song)));
        Self {
            title,
            body,
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

        let credit = if last_page {
            Some(authors_as_string(song))
        } else {
            None
        };

        let body = match item {
            LyricEntry::Verse { name, lines, .. } => {
                let line = &lines[lines_index];
                let mut body = "<p>".to_string();

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
                body += "</p>";
                body
            }
            LyricEntry::Instrument { name, .. } => {
                format!("<p>(instrumental {name})</p>")
            }
        };

        Self {
            title: None,
            body: Some(body),
            credit,
            theme,
        }
    }
}
