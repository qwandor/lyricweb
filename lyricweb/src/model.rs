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

    pub fn slides(&self) -> Vec<Slide<'_>> {
        let mut slides = Vec::new();
        for entry in &self.playlist {
            match entry {
                PlaylistEntry::Song { song_index } => {
                    let song = &self.songs[*song_index];
                    slides.push(Slide::SongStart {
                        song_index: *song_index,
                    });
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
    SongStart {
        song_index: usize,
    },
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

/// Returns the title to use for the given song.
pub fn title_for_song(song: &Song) -> &str {
    &song.properties.titles.titles[0].title
}

#[cfg(test)]
mod tests {
    use super::*;
    use openlyrics::types::{Lines, Lyrics, Properties};

    #[test]
    fn slides_empty() {
        let state = State {
            songs: vec![],
            playlist: vec![],
            current_slide: 0,
        };
        assert_eq!(state.slides(), vec![]);
    }

    #[test]
    fn slides_text() {
        let state = State {
            songs: vec![],
            playlist: vec![
                PlaylistEntry::Text("foo".to_string()),
                PlaylistEntry::Text("bar".to_string()),
            ],
            current_slide: 0,
        };
        assert_eq!(state.slides(), vec![Slide::Text("foo"), Slide::Text("bar")]);
    }

    #[test]
    fn slides_song() {
        let state = State {
            songs: vec![Song {
                properties: Properties::default(),
                lyrics: Lyrics {
                    lyrics: vec![
                        LyricEntry::Verse {
                            name: "v1".to_string(),
                            lang: None,
                            translit: None,
                            lines: vec![
                                Lines {
                                    break_optional: None,
                                    part: None,
                                    repeat: None,
                                    contents: vec![],
                                },
                                Lines {
                                    break_optional: None,
                                    part: None,
                                    repeat: None,
                                    contents: vec![],
                                },
                            ],
                        },
                        LyricEntry::Instrument {
                            name: "i1".to_string(),
                            lines: vec![],
                        },
                    ],
                },
            }],
            playlist: vec![PlaylistEntry::Song { song_index: 0 }],
            current_slide: 0,
        };
        assert_eq!(
            state.slides(),
            vec![
                Slide::SongStart { song_index: 0 },
                Slide::Lyrics {
                    song_index: 0,
                    lyric_entry_index: 0,
                    lines_index: 0,
                },
                Slide::Lyrics {
                    song_index: 0,
                    lyric_entry_index: 0,
                    lines_index: 1,
                },
                Slide::Lyrics {
                    song_index: 0,
                    lyric_entry_index: 1,
                    lines_index: 0,
                }
            ]
        );
    }
}
