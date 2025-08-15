// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use openlyrics::{LyricEntry, Lyrics, Song, VerseContent};
use quick_xml::de::from_reader;
use std::io::stdin;

fn main() {
    let song: Song = from_reader(stdin().lock()).unwrap();
    println!("{song:#?}");
    println!("= {} =", song.properties.titles.titles[0].title);
    print_lyrics(&song.lyrics);
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

fn simplify_contents(contents: &[VerseContent]) -> Vec<String> {
    let mut simple_lines = vec![];
    add_simple_contents(contents, &mut simple_lines);
    simple_lines
        .into_iter()
        .map(|line| line.trim().to_owned())
        .collect()
}

fn add_simple_contents(contents: &[VerseContent], simple_lines: &mut Vec<String>) {
    for content in contents {
        add_simple_content(content, simple_lines);
    }
}

fn add_simple_content(content: &VerseContent, simple_lines: &mut Vec<String>) {
    match content {
        VerseContent::Text(text) => {
            if simple_lines.is_empty() {
                simple_lines.push(String::new());
            }
            let text = text.replace(char::is_whitespace, " ");
            let line = simple_lines.last_mut().unwrap();
            line.push_str(text.trim());
            if text.ends_with(' ') {
                line.push(' ');
            }
        }
        VerseContent::Chord { contents, .. } => {
            add_simple_contents(contents, simple_lines);
        }
        VerseContent::Br => {
            simple_lines.push(String::new());
        }
        VerseContent::Comment(_) => {}
        VerseContent::Tag { contents, .. } => {
            add_simple_contents(contents, simple_lines);
        }
    }
}
