// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use clap::Parser;
use eyre::Report;
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Lyrics, Properties, Song},
};
use quick_xml::de::from_reader;
use std::{fs::File, io::BufReader, path::PathBuf};

fn main() -> Result<(), Report> {
    pretty_env_logger::init();

    match Args::parse() {
        Args::Print { path } => {
            let song: Song = from_reader(BufReader::new(File::open(path)?)).unwrap();
            print_header(&song.properties);
            print_lyrics(&song.lyrics);
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Parser)]
enum Args {
    /// Print the lyrics from the given OpenLyrics XML file to standard output.
    Print { path: PathBuf },
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
