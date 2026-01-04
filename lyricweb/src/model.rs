// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

pub mod helpers;
pub mod slide;

use self::helpers::title_for_song;
use openlyrics::types::{LyricEntry, Song};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct State {
    #[serde(default)]
    pub songs: BTreeMap<u32, Song>,
    #[serde(default)]
    pub playlists: BTreeMap<u32, Playlist>,
    #[serde(default)]
    pub theme: Theme,
}

impl Default for State {
    fn default() -> Self {
        Self {
            songs: Default::default(),
            playlists: [(0, Playlist::new("Playlist"))].into_iter().collect(),
            theme: Default::default(),
        }
    }
}

impl State {
    /// Returns a list of all songs, sorted by title.
    pub fn songs_by_title(&self) -> Vec<(&u32, &Song)> {
        let mut songs = self.songs.iter().collect::<Vec<_>>();
        songs.sort_by_key(|(_, song)| title_for_song(song));
        songs
    }

    /// Adds the given song to the database, and returns its ID.
    ///
    /// If the song already exists then the ID of the existing copy is returned without adding a
    /// duplicate.
    pub fn add_song(&mut self, song: Song) -> u32 {
        // No point adding an exact duplicate.
        if let Some((&existing_id, _)) = self
            .songs
            .iter()
            .find(|&(_, existing_song)| existing_song == &song)
        {
            return existing_id;
        }

        let id = self
            .songs
            .iter()
            .map(|(i, _)| i + 1)
            .max()
            .unwrap_or_default();
        self.songs.insert(id, song);
        id
    }

    pub fn add_playlist(&mut self, playlist: Playlist) -> u32 {
        let id = self
            .playlists
            .iter()
            .map(|(i, _)| i + 1)
            .max()
            .unwrap_or_default();
        self.playlists.insert(id, playlist);
        id
    }

    /// Remove the song with the given ID from the database, and replace any playlist entries
    /// referring to it with a text entry.
    pub fn remove_song(&mut self, id: u32) {
        for playlist in self.playlists.values_mut() {
            for entry in &mut playlist.entries {
                if matches!(entry, PlaylistEntry::Song { song_id } if *song_id == id) {
                    *entry = PlaylistEntry::Text("Song removed".to_string());
                }
            }
        }

        self.songs.remove(&id);
    }

    pub fn slide(&self, index: SlideIndex) -> Option<Slide<'_>> {
        let entry = self
            .playlists
            .get(&index.playlist_id)?
            .entries
            .get(index.entry_index)?;
        match entry {
            PlaylistEntry::Song { song_id } => {
                let song = &self.songs[song_id];
                if index.page_index == 0 {
                    Some(Slide::SongStart { song_id: *song_id })
                } else {
                    let mut index_left = index.page_index - 1;
                    if let Some(verse_order) = &song.properties.verse_order {
                        let verse_order_count = verse_order.split(' ').count();
                        for (verse_order_index, verse) in verse_order.split(' ').enumerate() {
                            if let Some((lyric_entry_index, lyric_entry)) = song
                                .lyrics
                                .lyrics
                                .iter()
                                .enumerate()
                                .find(|(_, lyric_entry)| lyric_entry.name() == verse)
                            {
                                match lyric_entry {
                                    LyricEntry::Verse { lines, .. } => {
                                        if index_left < lines.len() {
                                            return Some(Slide::Lyrics {
                                                song_id: *song_id,
                                                lyric_entry_index,
                                                lines_index: index_left,
                                                last_page: verse_order_index
                                                    == verse_order_count - 1
                                                    && index_left == lines.len() - 1,
                                            });
                                        } else {
                                            index_left -= lines.len();
                                        }
                                    }
                                    LyricEntry::Instrument { .. } => {
                                        if index_left == 0 {
                                            return Some(Slide::Lyrics {
                                                song_id: *song_id,
                                                lyric_entry_index,
                                                lines_index: 0,
                                                last_page: verse_order_index
                                                    == verse_order_count - 1,
                                            });
                                        } else {
                                            index_left -= 1;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        for (lyric_entry_index, item) in song.lyrics.lyrics.iter().enumerate() {
                            match item {
                                LyricEntry::Verse { lines, .. } => {
                                    if index_left < lines.len() {
                                        return Some(Slide::Lyrics {
                                            song_id: *song_id,
                                            lyric_entry_index,
                                            lines_index: index_left,
                                            last_page: lyric_entry_index
                                                == song.lyrics.lyrics.len() - 1
                                                && index_left == lines.len() - 1,
                                        });
                                    } else {
                                        index_left -= lines.len();
                                    }
                                }
                                LyricEntry::Instrument { .. } => {
                                    if index_left == 0 {
                                        return Some(Slide::Lyrics {
                                            song_id: *song_id,
                                            lyric_entry_index,
                                            lines_index: 0,
                                            last_page: lyric_entry_index
                                                == song.lyrics.lyrics.len() - 1,
                                        });
                                    } else {
                                        index_left -= 1;
                                    }
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

    pub fn slides_for_song(&self, song_id: u32) -> Vec<Slide<'_>> {
        let mut slides = Vec::new();
        let song = &self.songs[&song_id];
        slides.push(Slide::SongStart { song_id: song_id });
        if let Some(verse_order) = &song.properties.verse_order {
            let verse_order_count = verse_order.split(' ').count();
            for (verse_order_index, verse) in verse_order.split(' ').enumerate() {
                if let Some((lyric_entry_index, lyric_entry)) = song
                    .lyrics
                    .lyrics
                    .iter()
                    .enumerate()
                    .find(|(_, lyric_entry)| lyric_entry.name() == verse)
                {
                    push_lyric_entry_pages(
                        &mut slides,
                        lyric_entry_index,
                        lyric_entry,
                        song_id,
                        verse_order_index == verse_order_count - 1,
                    );
                }
            }
        } else {
            for (lyric_entry_index, lyric_entry) in song.lyrics.lyrics.iter().enumerate() {
                push_lyric_entry_pages(
                    &mut slides,
                    lyric_entry_index,
                    lyric_entry,
                    song_id,
                    lyric_entry_index == song.lyrics.lyrics.len() - 1,
                );
            }
        }
        slides
    }

    pub fn slides(&self, playlist_id: u32) -> Vec<(SlideIndex, Slide<'_>)> {
        let Some(playlist) = self.playlists.get(&playlist_id) else {
            return vec![];
        };
        let mut slides = Vec::new();
        for (entry_index, entry) in playlist.entries.iter().enumerate() {
            match entry {
                &PlaylistEntry::Song { song_id } => {
                    slides.extend(self.slides_for_song(song_id).into_iter().enumerate().map(
                        |(page_index, slide)| {
                            (
                                SlideIndex {
                                    playlist_id,
                                    entry_index,
                                    page_index,
                                },
                                slide,
                            )
                        },
                    ));
                }
                PlaylistEntry::Text(text) => slides.push((
                    SlideIndex {
                        playlist_id,
                        entry_index,
                        page_index: 0,
                    },
                    Slide::Text(text),
                )),
            }
        }
        slides
    }

    /// Merges the contents of the other state into this one.
    pub fn merge(&mut self, other: &State) {
        let mut other_song_ids_to_ours = BTreeMap::new();
        for (id, song) in &other.songs {
            other_song_ids_to_ours.insert(id, self.add_song(song.clone()));
        }

        for playlist in other.playlists.values() {
            let mut playlist = playlist.clone();
            // Update song IDs.
            for entry in &mut playlist.entries {
                if let PlaylistEntry::Song { song_id } = entry {
                    if let Some(&our_song_id) = other_song_ids_to_ours.get(song_id) {
                        *song_id = our_song_id;
                    } else {
                        *entry = PlaylistEntry::Text(format!("Invalid song ID {song_id}"));
                    }
                }
            }

            // Add it if we don't already have the exact same playlist.
            if !self
                .playlists
                .values()
                .any(|existing_playlist| existing_playlist == &playlist)
            {
                self.add_playlist(playlist);
            }
        }

        self.theme = other.theme.clone();
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Theme {
    #[serde(default)]
    pub heading_size: u32,
    #[serde(default)]
    pub body_size: u32,
    #[serde(default)]
    pub heading_colour: String,
    #[serde(default)]
    pub body_colour: String,
    #[serde(default)]
    pub background_colour: String,
    #[serde(default)]
    pub font_family: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            heading_size: 5,
            body_size: 4,
            heading_colour: "#000000".to_string(),
            body_colour: "#000000".to_string(),
            background_colour: "#ffffff".to_string(),
            font_family: "sans-serif".to_string(),
        }
    }
}

fn push_lyric_entry_pages(
    slides: &mut Vec<Slide>,
    lyric_entry_index: usize,
    lyric_entry: &LyricEntry,
    song_id: u32,
    last_entry: bool,
) {
    match lyric_entry {
        LyricEntry::Verse { lines, .. } => {
            slides.extend((0..lines.len()).map(|lines_index| Slide::Lyrics {
                song_id,
                lyric_entry_index,
                lines_index,
                last_page: last_entry && lines_index == lines.len() - 1,
            }));
        }
        LyricEntry::Instrument { .. } => {
            slides.push(Slide::Lyrics {
                song_id,
                lyric_entry_index,
                lines_index: 0,
                last_page: last_entry,
            });
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Playlist {
    pub name: String,
    #[serde(default)]
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    /// Moves the playlist entry containing the slide at the given index up or down by the given
    /// offset.
    ///
    /// Returns true if a change was made, or false if nothing was changed because the offset or
    /// slide was out of range.
    pub fn move_entry_index(&mut self, entry_index: usize, offset: isize) -> bool {
        if let Some(new_index) = entry_index.checked_add_signed(offset)
            && new_index < self.entries.len()
        {
            self.entries.swap(entry_index, new_index);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Slide<'a> {
    SongStart {
        song_id: u32,
    },
    Lyrics {
        song_id: u32,
        lyric_entry_index: usize,
        lines_index: usize,
        last_page: bool,
    },
    Text(&'a str),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlaylistEntry {
    Song { song_id: u32 },
    Text(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlideIndex {
    /// The ID of the playlist containing the slide.
    pub playlist_id: u32,
    /// The index of the song or text entry within the playlist.
    pub entry_index: usize,
    /// The index of the page within the entry.
    pub page_index: usize,
}

impl Display for SlideIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{}",
            self.playlist_id, self.entry_index, self.page_index
        )
    }
}

impl FromStr for SlideIndex {
    type Err = ParseSlideIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(',').collect::<Vec<_>>();
        let [playlist_id, entry_index, page_index] = parts.as_slice() else {
            return Err(ParseSlideIndexError::WrongNumberOfParts);
        };
        Ok(Self {
            playlist_id: playlist_id.parse()?,
            entry_index: entry_index.parse()?,
            page_index: page_index.parse()?,
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ParseSlideIndexError {
    #[error("Wrong number of parts")]
    WrongNumberOfParts,
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use openlyrics::types::{Lines, Lyrics, Properties};

    #[test]
    fn slides_empty() {
        let state = State::default();
        assert_eq!(state.slides(0), vec![]);
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 0,
                entry_index: 0,
                page_index: 0
            }),
            None
        );
    }

    #[test]
    fn slides_text() {
        let state = State {
            playlists: [(
                42,
                Playlist {
                    name: "Playlist".to_string(),
                    entries: vec![
                        PlaylistEntry::Text("foo".to_string()),
                        PlaylistEntry::Text("bar".to_string()),
                    ],
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        assert_eq!(
            state.slides(42),
            vec![
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 0,
                    },
                    Slide::Text("foo")
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 1,
                        page_index: 0,
                    },
                    Slide::Text("bar")
                )
            ]
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 0,
                page_index: 0,
            }),
            Some(Slide::Text("foo"))
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 0,
                page_index: 1,
            }),
            None
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 1,
                page_index: 0,
            }),
            Some(Slide::Text("bar"))
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 2,
                page_index: 0,
            }),
            None
        );
    }

    #[test]
    fn slides_song() {
        let state = State {
            songs: [(
                0,
                Song {
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
                },
            )]
            .into_iter()
            .collect(),
            playlists: [(
                42,
                Playlist {
                    name: "Playlist".to_string(),
                    entries: vec![PlaylistEntry::Song { song_id: 0 }],
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        assert_eq!(
            state.slides(42),
            vec![
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 0,
                    },
                    Slide::SongStart { song_id: 0 }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 1,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 0,
                        lines_index: 0,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 2,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 0,
                        lines_index: 1,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 3,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 1,
                        lines_index: 0,
                        last_page: true,
                    }
                ),
            ]
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 0,
                page_index: 0,
            }),
            Some(Slide::SongStart { song_id: 0 })
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 0,
                page_index: 1,
            }),
            Some(Slide::Lyrics {
                song_id: 0,
                lyric_entry_index: 0,
                lines_index: 0,
                last_page: false,
            })
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 0,
                page_index: 4,
            }),
            None
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 1,
                page_index: 0,
            }),
            None
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 1,
                page_index: 1,
            }),
            None
        );
    }

    #[test]
    fn slides_verse_order() {
        let state = State {
            songs: [(
                0,
                Song {
                    properties: Properties {
                        verse_order: Some("v1 c v2 c v3 c".to_string()),
                        ..Default::default()
                    },
                    lyrics: Lyrics {
                        lyrics: vec![
                            LyricEntry::Verse {
                                name: "v1".to_string(),
                                lang: None,
                                translit: None,
                                lines: vec![Lines {
                                    break_optional: None,
                                    part: None,
                                    repeat: None,
                                    contents: vec![],
                                }],
                            },
                            LyricEntry::Verse {
                                name: "c".to_string(),
                                lang: None,
                                translit: None,
                                lines: vec![Lines {
                                    break_optional: None,
                                    part: None,
                                    repeat: None,
                                    contents: vec![],
                                }],
                            },
                            LyricEntry::Verse {
                                name: "v2".to_string(),
                                lang: None,
                                translit: None,
                                lines: vec![Lines {
                                    break_optional: None,
                                    part: None,
                                    repeat: None,
                                    contents: vec![],
                                }],
                            },
                            LyricEntry::Verse {
                                name: "v3".to_string(),
                                lang: None,
                                translit: None,
                                lines: vec![Lines {
                                    break_optional: None,
                                    part: None,
                                    repeat: None,
                                    contents: vec![],
                                }],
                            },
                        ],
                    },
                },
            )]
            .into_iter()
            .collect(),
            playlists: [(
                42,
                Playlist {
                    name: "Playlist".to_string(),
                    entries: vec![PlaylistEntry::Song { song_id: 0 }],
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        assert_eq!(
            state.slides(42),
            vec![
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 0,
                    },
                    Slide::SongStart { song_id: 0 }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 1,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 0,
                        lines_index: 0,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 2,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 1,
                        lines_index: 0,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 3,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 2,
                        lines_index: 0,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 4,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 1,
                        lines_index: 0,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 5,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 3,
                        lines_index: 0,
                        last_page: false,
                    }
                ),
                (
                    SlideIndex {
                        playlist_id: 42,
                        entry_index: 0,
                        page_index: 6,
                    },
                    Slide::Lyrics {
                        song_id: 0,
                        lyric_entry_index: 1,
                        lines_index: 0,
                        last_page: true,
                    }
                ),
            ]
        );
        assert_eq!(
            state.slide(SlideIndex {
                playlist_id: 42,
                entry_index: 0,
                page_index: 4,
            }),
            Some(Slide::Lyrics {
                song_id: 0,
                lyric_entry_index: 1,
                lines_index: 0,
                last_page: false,
            })
        );
    }
}
