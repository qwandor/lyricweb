// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use openlyrics::types::{LyricEntry, Song};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct State {
    pub songs: Vec<Song>,
    pub playlist: Vec<PlaylistEntry>,
}

impl State {
    pub const fn new() -> Self {
        Self {
            songs: Vec::new(),
            playlist: Vec::new(),
        }
    }

    pub fn slide(&self, index: SlideIndex) -> Option<Slide<'_>> {
        let entry = self.playlist.get(index.entry_index)?;
        match entry {
            PlaylistEntry::Song { song_index } => {
                let song = &self.songs[*song_index];
                if index.page_index == 0 {
                    Some(Slide::SongStart {
                        song_index: *song_index,
                    })
                } else {
                    let mut index_left = index.page_index - 1;
                    for (lyric_entry_index, item) in song.lyrics.lyrics.iter().enumerate() {
                        match item {
                            LyricEntry::Verse { lines, .. } => {
                                if index_left < lines.len() {
                                    return Some(Slide::Lyrics {
                                        song_index: *song_index,
                                        lyric_entry_index,
                                        lines_index: index_left,
                                    });
                                } else {
                                    index_left -= lines.len();
                                }
                            }
                            LyricEntry::Instrument { .. } => {
                                if index_left == 0 {
                                    return Some(Slide::Lyrics {
                                        song_index: *song_index,
                                        lyric_entry_index,
                                        lines_index: 0,
                                    });
                                } else {
                                    index_left -= 1;
                                }
                            }
                        }
                    }
                    None
                }
            }
            PlaylistEntry::Text(text) => {
                if index.page_index == 0 {
                    Some(Slide::Text(text))
                } else {
                    None
                }
            }
        }
    }

    pub fn slides(&self) -> Vec<(SlideIndex, Slide<'_>)> {
        let mut slides = Vec::new();
        for (entry_index, entry) in self.playlist.iter().enumerate() {
            match entry {
                PlaylistEntry::Song { song_index } => {
                    let song = &self.songs[*song_index];
                    slides.push((
                        SlideIndex {
                            entry_index,
                            page_index: 0,
                        },
                        Slide::SongStart {
                            song_index: *song_index,
                        },
                    ));
                    let mut page_index = 1;
                    for (lyric_entry_index, item) in song.lyrics.lyrics.iter().enumerate() {
                        match item {
                            LyricEntry::Verse { lines, .. } => {
                                for lines_index in 0..lines.len() {
                                    slides.push((
                                        SlideIndex {
                                            entry_index,
                                            page_index,
                                        },
                                        Slide::Lyrics {
                                            song_index: *song_index,
                                            lyric_entry_index,
                                            lines_index,
                                        },
                                    ));
                                    page_index += 1;
                                }
                            }
                            LyricEntry::Instrument { .. } => {
                                slides.push((
                                    SlideIndex {
                                        entry_index,
                                        page_index,
                                    },
                                    Slide::Lyrics {
                                        song_index: *song_index,
                                        lyric_entry_index,
                                        lines_index: 0,
                                    },
                                ));
                                page_index += 1;
                            }
                        }
                    }
                }
                PlaylistEntry::Text(text) => slides.push((
                    SlideIndex {
                        entry_index,
                        page_index: 0,
                    },
                    Slide::Text(text),
                )),
            }
        }
        slides
    }

    /// Returns the `SlideIndex` for the given overall slide index.
    pub fn slide_index_for_index(&self, mut slide_index: usize) -> Option<SlideIndex> {
        for (i, entry) in self.playlist.iter().enumerate() {
            let entry_length = match entry {
                PlaylistEntry::Song { song_index } => {
                    let song = &self.songs[*song_index];
                    1 + song
                        .lyrics
                        .lyrics
                        .iter()
                        .map(|item| match item {
                            LyricEntry::Verse { lines, .. } => lines.len(),
                            LyricEntry::Instrument { .. } => 1,
                        })
                        .sum::<usize>()
                }
                PlaylistEntry::Text(_) => 1,
            };
            if slide_index < entry_length {
                return Some(SlideIndex {
                    entry_index: i,
                    page_index: slide_index,
                });
            } else {
                slide_index -= entry_length;
            }
        }
        None
    }

    /// Moves the playlist entry containing the slide at the given index up or down by the given
    /// offset.
    ///
    /// Returns true if a change was made, or false if nothing was changed because the offset or
    /// slide was out of range.
    pub fn move_entry_index(&mut self, entry_index: usize, offset: isize) -> bool {
        if let Some(new_index) = entry_index.checked_add_signed(offset)
            && new_index < self.playlist.len()
        {
            self.playlist.swap(entry_index, new_index);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlaylistEntry {
    Song { song_index: usize },
    Text(String),
}

/// Returns the title to use for the given song.
pub fn title_for_song(song: &Song) -> &str {
    &song.properties.titles.titles[0].title
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlideIndex {
    /// The index of the song or text entry within the playlist.
    pub entry_index: usize,
    /// The index of the page within the entry.
    pub page_index: usize,
}

impl Display for SlideIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{},{}", self.entry_index, self.page_index)
    }
}

impl FromStr for SlideIndex {
    type Err = ParseSlideIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entry_index, page_index) = s
            .split_once(',')
            .ok_or(ParseSlideIndexError::MissingComma)?;
        Ok(Self {
            entry_index: entry_index.parse()?,
            page_index: page_index.parse()?,
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ParseSlideIndexError {
    #[error("Missing comma")]
    MissingComma,
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
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
        };
        assert_eq!(state.slides(), vec![]);
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 0,
                page_index: 0
            }),
            None
        );
    }

    #[test]
    fn slides_text() {
        let state = State {
            songs: vec![],
            playlist: vec![
                PlaylistEntry::Text("foo".to_string()),
                PlaylistEntry::Text("bar".to_string()),
            ],
        };
        assert_eq!(
            state.slides(),
            vec![
                (
                    SlideIndex {
                        entry_index: 0,
                        page_index: 0,
                    },
                    Slide::Text("foo")
                ),
                (
                    SlideIndex {
                        entry_index: 1,
                        page_index: 0,
                    },
                    Slide::Text("bar")
                )
            ]
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 0,
                page_index: 0,
            }),
            Some(Slide::Text("foo"))
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 0,
                page_index: 1,
            }),
            None
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 1,
                page_index: 0,
            }),
            Some(Slide::Text("bar"))
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 2,
                page_index: 0,
            }),
            None
        );
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
        };
        assert_eq!(
            state.slides(),
            vec![
                (
                    SlideIndex {
                        entry_index: 0,
                        page_index: 0,
                    },
                    Slide::SongStart { song_index: 0 }
                ),
                (
                    SlideIndex {
                        entry_index: 0,
                        page_index: 1,
                    },
                    Slide::Lyrics {
                        song_index: 0,
                        lyric_entry_index: 0,
                        lines_index: 0,
                    }
                ),
                (
                    SlideIndex {
                        entry_index: 0,
                        page_index: 2,
                    },
                    Slide::Lyrics {
                        song_index: 0,
                        lyric_entry_index: 0,
                        lines_index: 1,
                    }
                ),
                (
                    SlideIndex {
                        entry_index: 0,
                        page_index: 3,
                    },
                    Slide::Lyrics {
                        song_index: 0,
                        lyric_entry_index: 1,
                        lines_index: 0,
                    }
                ),
            ]
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 0,
                page_index: 0,
            }),
            Some(Slide::SongStart { song_index: 0 })
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 0,
                page_index: 1,
            }),
            Some(Slide::Lyrics {
                song_index: 0,
                lyric_entry_index: 0,
                lines_index: 0,
            })
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 0,
                page_index: 4,
            }),
            None
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 1,
                page_index: 0,
            }),
            None
        );
        assert_eq!(
            state.slide(SlideIndex {
                entry_index: 1,
                page_index: 1,
            }),
            None
        );
    }

    #[test]
    fn find_entry() {
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
            playlist: vec![
                PlaylistEntry::Song { song_index: 0 },
                PlaylistEntry::Text("Text".to_string()),
                PlaylistEntry::Song { song_index: 0 },
            ],
        };

        assert_eq!(
            state.slide_index_for_index(0),
            Some(SlideIndex {
                entry_index: 0,
                page_index: 0,
            })
        );
        assert_eq!(
            state.slide_index_for_index(1),
            Some(SlideIndex {
                entry_index: 0,
                page_index: 1,
            })
        );
        assert_eq!(
            state.slide_index_for_index(2),
            Some(SlideIndex {
                entry_index: 0,
                page_index: 2,
            })
        );
        assert_eq!(
            state.slide_index_for_index(3),
            Some(SlideIndex {
                entry_index: 0,
                page_index: 3,
            })
        );
        assert_eq!(
            state.slide_index_for_index(4),
            Some(SlideIndex {
                entry_index: 1,
                page_index: 0,
            })
        );
        assert_eq!(
            state.slide_index_for_index(5),
            Some(SlideIndex {
                entry_index: 2,
                page_index: 0,
            })
        );
    }
}
