// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use openlyrics::{
    simplify_contents,
    types::{Author, Lines, LyricEntry, Song, Songbook, VerseContent},
};
use std::fmt::Write;

/// Returns the title to use for the given song.
pub fn title_for_song(song: &Song) -> &str {
    &song.properties.titles.titles[0].title
}

/// Returns the title to use for the given song, including the first songbook entry if there is one.
pub fn title_with_songbook(song: &Song) -> String {
    let title = title_for_song(song);
    if let Some(songbook_entry) = first_songbook(song) {
        format!("{songbook_entry}: {title}")
    } else {
        title.to_string()
    }
}

/// Returns the first songbook entry for the song formatted as a string, if there is one.
pub fn first_songbook(song: &Song) -> Option<String> {
    let songbook = song.properties.songbooks.songbooks.get(0)?;
    Some(songbook_to_string(songbook))
}

/// Returns whether the given song should be displayed when the given search filter is entered.
pub fn song_matches_filter(song: &Song, filter: &str) -> bool {
    title_with_songbook(song)
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

            let mut first_lines = true;
            for lines_entry in lines {
                if first_lines {
                    first_lines = false;
                } else {
                    writeln!(&mut text).unwrap();
                }

                for line in simplify_contents(&lines_entry.contents) {
                    writeln!(&mut text, "{line}").unwrap();
                }
            }
        }
    }
    text
}

/// Sets the lyrics of the given song by parsing a single string, of the format returned by
/// `lyrics_as_text`.
pub fn set_lyrics_from_text(song: &mut Song, text: &str) {
    let mut lines = text.lines().peekable();
    let mut lyrics = Vec::new();
    while let Some(line) = lines.peek() {
        let line = line.trim();
        let verse_name = if line.ends_with(':') {
            lines.next();
            line.trim_end_matches(':')
        } else {
            ""
        };

        // A verse may have several pages of lines.
        let mut verse_lines = Vec::new();
        while let Some(line) = lines.peek()
            && !line.trim().ends_with(':')
        {
            verse_lines.push(parse_lines(&mut lines));
        }

        lyrics.push(LyricEntry::Verse {
            name: verse_name.to_string(),
            lang: None,
            translit: None,
            lines: verse_lines,
        });
    }
    song.lyrics.lyrics = lyrics;
}

fn parse_lines<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Lines {
    let mut contents = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            break;
        }

        if !contents.is_empty() {
            contents.push(VerseContent::Br);
        }
        contents.push(VerseContent::Text(line.to_string()));
    }
    Lines {
        break_optional: None,
        part: None,
        repeat: None,
        contents,
    }
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

fn songbook_to_string(songbook: &Songbook) -> String {
    let name = &songbook.name;
    if let Some(entry) = &songbook.entry {
        format!("{name} {entry}")
    } else {
        format!("{name}")
    }
}

/// Returns the songbooks and entry numbers of the song as a single string, for displaying and
/// editing.
pub fn songbook_entries_as_string(song: &Song) -> String {
    song.properties
        .songbooks
        .songbooks
        .iter()
        .map(songbook_to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Sets the songbooks of the given song from a string of the format returned by
/// `songbook_entries_as_string`.
pub fn set_songbooks_from_string(song: &mut Song, songbook_entries: &str) {
    song.properties.songbooks.songbooks = songbook_entries
        .split(',')
        .filter_map(|songbook_entry| {
            let songbook_entry = songbook_entry.trim();
            if songbook_entry.is_empty() {
                None
            } else if let Some((name, entry)) = songbook_entry.rsplit_once(' ') {
                Some(Songbook {
                    name: name.to_string(),
                    entry: Some(entry.to_string()),
                })
            } else {
                Some(Songbook {
                    name: songbook_entry.to_string(),
                    entry: None,
                })
            }
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lyrics_and_back() {
        let mut song = Song::default();

        let text = "\
v1:
Some line
Another line.

More of verse 1

c:
Chorus, I guess

v2:
Verse 2 line

Some more lines
Line
";

        set_lyrics_from_text(&mut song, text);
        assert_eq!(lyrics_as_text(&song), text);
    }

    #[test]
    fn no_songbook() {
        let mut song = Song::default();
        assert_eq!(songbook_entries_as_string(&song), "");
        set_songbooks_from_string(&mut song, "");
        assert_eq!(song.properties.songbooks.songbooks, []);
    }
}
