// Copyright 2026 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use std::collections::BTreeMap;

use musicxml::{
    datatypes::Syllabic,
    elements::{LyricContents, MeasureElement, PartElement, ScorePartwise},
};
use openlyrics::types::{Author, LyricEntry, Song, Title};

use crate::lines_to_open_lyrics;

pub fn music_xml_to_open_lyrics(score: &ScorePartwise) -> Song {
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
