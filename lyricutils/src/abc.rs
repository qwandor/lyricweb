// Copyright 2026 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::lines_to_open_lyrics;
use abc_parser::datatypes::{
    Comment, HeaderLine, IgnoredLine, InfoField, LyricLine, LyricSymbol, SymbolAlignment, TuneBook,
    TuneLine,
};
use log::info;
use openlyrics::types::{Author, LyricEntry, Song, Theme, Title};
use regex::Regex;

pub fn tunebook_to_open_lyrics(tunebook: &TuneBook) -> Song {
    let verse_number_regex = Regex::new(r"^[1-9][0-9]?\.").unwrap();

    let mut song = Song::default();
    for comment in &tunebook.comments {
        if let IgnoredLine::Comment(Comment::CommentLine(_, comment)) = comment {
            if let Some((first, rest)) = comment.split_once(' ') {
                match first {
                    "OHAUTHOR" => {
                        song.properties
                            .authors
                            .authors
                            .extend(abc_author("words", rest));
                    }
                    "OHCOMPOSER" | "OHARRANGER" => {
                        song.properties
                            .authors
                            .authors
                            .extend(abc_author("music", rest));
                    }
                    "OHTRANSLATOR" => {
                        song.properties
                            .authors
                            .authors
                            .extend(abc_author("translation", rest));
                    }
                    "OHCATEGORY" => {
                        song.properties.themes.themes.push(Theme {
                            title: rest.to_lowercase(),
                            ..Default::default()
                        });
                    }
                    "OHTOPICS" => {
                        // TODO: Parse and convert to themes.
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(tune) = tunebook.tunes.get(0) {
        for header_line in &tune.header.lines {
            if let HeaderLine::Field(InfoField(name, value), _) = header_line {
                let value = value.trim();
                info!("{name}: {value:?}");
                match name {
                    'T' => {
                        song.properties.titles.titles.push(Title {
                            title: value.to_string(),
                            ..Default::default()
                        });
                    }
                    'M' => {
                        song.properties.time_signature = Some(value.to_string());
                    }
                    _ => {}
                }
            }
        }

        if let Some(body) = &tune.body {
            let mut verses = vec![];
            let mut verse = 0;
            let mut stylesheettext = false;
            for line in &body.lines {
                match line {
                    TuneLine::Music(_) => {
                        verse = 0;
                    }
                    TuneLine::Lyric(lyric_line) => {
                        let lyric = lyric_line_to_string(lyric_line);
                        if verses.len() < verse + 1 {
                            verses.push(Vec::new());
                        }
                        verses[verse].push(lyric);
                        verse += 1;
                    }
                    TuneLine::Comment(Comment::StylesheetDirective(directive)) => {
                        if directive.starts_with("begintext") {
                            stylesheettext = true;
                            verses.push(Vec::new());
                        } else if directive.starts_with("endtext") {
                            stylesheettext = false
                        } else if stylesheettext {
                            let directive = directive.trim();
                            if directive.is_empty() {
                                verses.push(Vec::new());
                            } else {
                                let line = directive.replace("\\t", "");
                                verses.last_mut().unwrap().push(line);
                            }
                        }
                    }
                    TuneLine::Symbol(_) => {}
                    TuneLine::Comment(_) => {}
                }
            }

            let mut chorus = None;
            if let Some(rest_max_line_count) = verses.iter().skip(1).map(|lines| lines.len()).max()
            {
                // If the first verse is longer than the rest, the end of it is probably a chorus.
                if verses[0].len() > rest_max_line_count {
                    let lines = verses[0].split_off(rest_max_line_count);
                    chorus = Some(make_verse("c".to_string(), lines));
                }
            }

            song.lyrics.lyrics = verses
                .into_iter()
                .enumerate()
                .map(|(i, mut verse)| {
                    verse[0] = verse_number_regex.replace(&verse[0], "").to_string();
                    make_verse(format!("v{}", i + 1), verse)
                })
                .collect();
            if let Some(chorus) = chorus {
                song.lyrics.lyrics.insert(1, chorus);
            }
        }
    }

    song
}

fn make_verse(name: String, lines: Vec<String>) -> LyricEntry {
    let lines = vec![lines_to_open_lyrics(lines)];
    LyricEntry::Verse {
        name,
        lines,
        lang: None,
        translit: None,
    }
}

fn lyric_line_to_string(lyric_line: &LyricLine) -> String {
    let mut line = String::new();
    let mut include_space = false;
    info!("{:?}", lyric_line.symbols);
    for symbol in &lyric_line.symbols {
        match symbol {
            LyricSymbol::Syllable(syllable) => {
                line += syllable;
                include_space = true;
            }
            LyricSymbol::Space(_) => {
                if include_space {
                    line += " ";
                    include_space = false;
                }
            }
            LyricSymbol::SymbolAlignment(SymbolAlignment::Break) => {
                include_space = false;
            }
            _ => {}
        }
    }
    if !include_space && !line.ends_with(" ") && !line.is_empty() {
        line += "-";
    }
    line.trim().to_string()
}

fn abc_author(author_type: &str, name: &str) -> Option<Author> {
    if name == "none" {
        None
    } else {
        Some(Author {
            author_type: Some(author_type.to_string()),
            lang: None,
            name: name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abc_parser::datatypes::SymbolAlignment;

    #[test]
    fn lyric_line_empty() {
        assert_eq!(lyric_line_to_string(&LyricLine::new(vec![])), "");
    }

    #[test]
    fn lyric_line_spaces() {
        assert_eq!(
            lyric_line_to_string(&LyricLine::new(vec![
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("foo".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Skip),
                LyricSymbol::Syllable("bar".to_string()),
                LyricSymbol::Space("  ".to_string()),
                LyricSymbol::Syllable("ba".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Break),
                LyricSymbol::Space("  ".to_string()),
                LyricSymbol::Syllable("z".to_string()),
                LyricSymbol::Space(" ".to_string()),
            ])),
            "foobar baz"
        );
    }
}
