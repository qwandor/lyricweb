// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use abc_parser::{
    abc::tune_book,
    datatypes::{
        Comment, HeaderLine, IgnoredLine, InfoField, LyricLine, LyricSymbol, SymbolAlignment,
        TuneBook, TuneLine,
    },
};
use clap::{Parser, ValueEnum};
use eyre::{OptionExt, Report, eyre};
use log::info;
use musicxml::{
    datatypes::Syllabic,
    elements::{LyricContents, MeasureElement, PartElement, ScorePartwise},
    read_score_partwise,
};
use openlyrics::{
    simplify_contents,
    types::{Author, Lines, LyricEntry, Lyrics, Properties, Song, Theme, Title, VerseContent},
};
use quick_xml::de::from_reader;
use std::{
    collections::BTreeMap,
    fmt::Debug,
    fs::{File, read_to_string},
    io::BufReader,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Report> {
    pretty_env_logger::init();

    match Args::parse() {
        Args::Print { input_format, path } => {
            let song = read_and_convert(&path, input_format)?;
            print_header(&song.properties);
            print_lyrics(&song.lyrics);
        }
    }

    Ok(())
}

/// Reads from the given file in the given format, and converts it to OpenLyrics format.
fn read_and_convert(path: &Path, format: Format) -> Result<Song, Report> {
    Ok(match format {
        Format::Abc => {
            let tunebook = tune_book(&read_to_string(path)?)?;
            tunebook_to_open_lyrics(&tunebook)
        }
        Format::MusicXml => {
            let score =
                read_score_partwise(path.to_str().ok_or_eyre("Path is not a valid string")?)
                    .map_err(|e| eyre!("{e}"))?;
            music_xml_to_open_lyrics(&score)
        }
        Format::OpenLyrics => from_reader(BufReader::new(File::open(path)?))?,
    })
}

#[derive(Clone, Debug, Parser)]
enum Args {
    /// Print the lyrics from the given file to standard output.
    Print {
        /// Format of the input file.
        #[arg(long)]
        input_format: Format,
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Format {
    Abc,
    MusicXml,
    OpenLyrics,
}

fn print_header(properties: &Properties) {
    println!("= {} =", properties.titles.titles[0].title);
    for author in &properties.authors.authors {
        if let Some(author_type) = &author.author_type {
            println!("Author ({author_type}): {}", author.name);
        } else {
            println!("Author: {}", author.name);
        }
    }
}

fn print_lyrics(lyrics: &Lyrics) {
    for item in &lyrics.lyrics {
        match item {
            LyricEntry::Verse { name, lines, .. } => {
                println!("{name}:");
                for line in lines {
                    if let Some(part) = &line.part {
                        println!("({part})");
                    }
                    for simple_line in &simplify_contents(&line.contents) {
                        println!("{simple_line}");
                    }
                    if let Some(repeat) = line.repeat {
                        println!("x{repeat}");
                    }
                    println!();
                }
            }
            LyricEntry::Instrument { name, .. } => println!("Skipping instrumental {name}."),
        }
    }
}

fn music_xml_to_open_lyrics(score: &ScorePartwise) -> Song {
    let mut song = Song::default();

    if let Some(work) = &score.content.work {
        if let Some(title) = &work.content.work_title {
            song.properties.titles.titles.push(Title {
                title: title.content.clone(),
                ..Default::default()
            });
        }
    }
    if let Some(movement_title) = &score.content.movement_title {
        song.properties.titles.titles.push(Title {
            title: movement_title.content.clone(),
            ..Default::default()
        });
    }
    if let Some(identification) = &score.content.identification {
        for creator in &identification.content.creator {
            song.properties.authors.authors.push(Author {
                author_type: creator
                    .attributes
                    .r#type
                    .as_ref()
                    .map(|token| token.0.clone()),
                name: creator.content.clone(),
                ..Default::default()
            });
        }
    }

    let mut lyrics = BTreeMap::<String, Vec<String>>::new();
    for part in &score.content.part {
        for part_element in &part.content {
            match part_element {
                PartElement::Measure(measure) => {
                    for measure_element in &measure.content {
                        match measure_element {
                            MeasureElement::Note(note) => {
                                for lyric in &note.content.lyric {
                                    let verse_number = lyric
                                        .attributes
                                        .number
                                        .as_ref()
                                        .map(|number| number.0.clone())
                                        .unwrap_or_default();
                                    match &lyric.content {
                                        LyricContents::Text(text_lyric) => {
                                            let entry = lyrics.entry(verse_number).or_default();
                                            if entry.is_empty() {
                                                entry.push("".to_string());
                                            }
                                            let last_line = entry.last_mut().unwrap();
                                            if let Some(syllabic) = &text_lyric.syllabic {
                                                if !last_line.is_empty()
                                                    && matches!(
                                                        syllabic.content,
                                                        Syllabic::Begin | Syllabic::Single
                                                    )
                                                {
                                                    last_line.push_str(" ");
                                                }
                                            }
                                            last_line.push_str(
                                                &text_lyric
                                                    .text
                                                    .content
                                                    .replace("&quot;", "\"")
                                                    .replace("&apos;", "'"),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    // End of measure, start a new line for each verse.
                    for verse in lyrics.values_mut() {
                        verse.push("".to_string());
                    }
                }
                _ => {}
            }
        }
    }

    song.lyrics.lyrics = lyrics
        .into_iter()
        .map(|(verse, verse_lyrics)| LyricEntry::Verse {
            name: format!("v{verse}"),
            lang: None,
            translit: None,
            lines: vec![lines_to_open_lyrics(verse_lyrics)],
        })
        .collect();

    song
}

fn lines_to_open_lyrics(verse_lyrics: Vec<String>) -> Lines {
    let mut contents = Vec::new();
    for line in verse_lyrics {
        if line.is_empty() {
            continue;
        }
        if !contents.is_empty() {
            contents.push(VerseContent::Br);
        }
        contents.push(VerseContent::Text(line));
    }
    Lines {
        contents,
        ..Default::default()
    }
}

fn tunebook_to_open_lyrics(tunebook: &TuneBook) -> Song {
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
                    TuneLine::Symbol(_) => {}
                    TuneLine::Comment(_) => {}
                }
            }
            song.lyrics.lyrics = verses
                .into_iter()
                .enumerate()
                .map(|(i, verse)| {
                    let mut contents = Vec::new();
                    for (i, line) in verse.into_iter().enumerate() {
                        if i != 0 {
                            contents.push(VerseContent::Br);
                        }
                        contents.push(VerseContent::Text(line));
                    }
                    LyricEntry::Verse {
                        name: format!("v{}", i + 1),
                        lines: vec![Lines {
                            contents,
                            ..Default::default()
                        }],
                        lang: None,
                        translit: None,
                    }
                })
                .collect();
        }
    }

    song
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
    use clap::CommandFactory;

    #[test]
    fn verify_args() {
        Args::command().debug_assert();
    }

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
