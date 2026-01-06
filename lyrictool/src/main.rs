// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use abc_parser::abc::tune_book;
use clap::{Parser, ValueEnum};
use eyre::{OptionExt, Report, eyre};
use lyricutils::{music_xml_to_open_lyrics, tunebook_to_open_lyrics};
use musicxml::read_score_partwise;
use openlyrics::{
    simplify_contents,
    types::{LyricEntry, Lyrics, Properties, Song},
};
use quick_xml::de::from_reader;
use std::{
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_args() {
        Args::command().debug_assert();
    }
}
