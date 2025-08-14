use openlyrics::{LyricEntry, Lyrics, Song, VerseContent};
use quick_xml::{de::from_reader, se::to_string};
use std::io::stdin;

fn main() {
    let song: Song = from_reader(stdin().lock()).unwrap();
    println!("{song:#?}");
    println!("= {} =", song.properties.titles.titles[0].title);
    print_lyrics(&song.lyrics);
    let xml = to_string(&song).unwrap();
    println!("{xml}");
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
                    print_contents(&line.contents);
                    println!();
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

fn print_contents(contents: &[VerseContent]) {
    for content in contents {
        print_content(content);
    }
}

fn print_content(content: &VerseContent) {
    match content {
        VerseContent::Text(text) => {
            let text = text.replace(char::is_whitespace, " ");
            print!("{}", text.trim());
            if text.ends_with(' ') {
                print!(" ");
            }
        }
        VerseContent::Chord { contents, .. } => {
            print_contents(contents);
        }
        VerseContent::Br => println!(),
        VerseContent::Comment(_) => {}
        VerseContent::Tag { contents, .. } => {
            print_contents(contents);
        }
    }
}
