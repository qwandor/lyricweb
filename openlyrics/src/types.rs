// Copyright 2025 The lyricweb Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "song")]
pub struct Song {
    pub properties: Properties,
    pub lyrics: Lyrics,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub titles: Titles,
    #[serde(default, skip_serializing_if = "Authors::is_empty")]
    pub authors: Authors,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ccli_no: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transposition: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<Tempo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    // TODO: Parse space-separated values into a Vec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verse_order: Option<String>,
    #[serde(default, skip_serializing_if = "Songbooks::is_empty")]
    pub songbooks: Songbooks,
    #[serde(default, skip_serializing_if = "Themes::is_empty")]
    pub themes: Themes,
    #[serde(default, skip_serializing_if = "Comments::is_empty")]
    pub comments: Comments,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Titles {
    #[serde(rename = "title")]
    pub titles: Vec<Title>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Title {
    #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@translit", skip_serializing_if = "Option::is_none")]
    pub translit: Option<String>,
    #[serde(rename = "@original", skip_serializing_if = "Option::is_none")]
    pub original: Option<bool>,
    #[serde(rename = "$text")]
    pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Authors {
    #[serde(rename = "author")]
    pub authors: Vec<Author>,
}

impl Authors {
    pub fn is_empty(&self) -> bool {
        self.authors.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Author {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub author_type: Option<String>,
    #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "@type", content = "$text", rename_all = "lowercase")]
pub enum Tempo {
    Bpm(u16),
    Text(String),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Songbooks {
    #[serde(rename = "songbook")]
    pub songbooks: Vec<Songbook>,
}

impl Songbooks {
    pub fn is_empty(&self) -> bool {
        self.songbooks.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Songbook {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@entry", skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Themes {
    #[serde(rename = "theme")]
    pub themes: Vec<Theme>,
}

impl Themes {
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Theme {
    #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@translit", skip_serializing_if = "Option::is_none")]
    pub translit: Option<String>,
    #[serde(rename = "$text")]
    pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Comments {
    #[serde(rename = "comment")]
    pub comments: Vec<String>,
}

impl Comments {
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lyrics {
    #[serde(rename = "$value", default)]
    pub lyrics: Vec<LyricEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LyricEntry {
    Verse {
        #[serde(rename = "@name")]
        name: String,
        #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
        lang: Option<String>,
        #[serde(rename = "@translit", skip_serializing_if = "Option::is_none")]
        translit: Option<String>,
        #[serde(default)]
        lines: Vec<Lines>,
    },
    Instrument {
        #[serde(rename = "@name")]
        name: String,
        #[serde(default)]
        lines: Vec<InstrumentLines>,
    },
}

impl LyricEntry {
    /// Returns the name of the entry, no matter whether it's a verse or instrumental.
    pub fn name(&self) -> &str {
        match self {
            LyricEntry::Verse { name, .. } => name,
            LyricEntry::Instrument { name, .. } => name,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lines {
    #[serde(rename = "@break", skip_serializing_if = "Option::is_none")]
    pub break_optional: Option<String>,
    #[serde(rename = "@part", skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    #[serde(rename = "@repeat", skip_serializing_if = "Option::is_none")]
    pub repeat: Option<u32>,
    #[serde(rename = "$value", default)]
    pub contents: Vec<VerseContent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VerseContent {
    #[serde(rename = "$text")]
    Text(String),
    Chord {
        #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(rename = "@root", skip_serializing_if = "Option::is_none")]
        root: Option<String>,
        #[serde(rename = "@bass", skip_serializing_if = "Option::is_none")]
        bass: Option<String>,
        #[serde(rename = "@structure", skip_serializing_if = "Option::is_none")]
        structure: Option<String>,
        #[serde(rename = "@upbeat", skip_serializing_if = "Option::is_none")]
        upbeat: Option<bool>,
        #[serde(rename = "$value", default)]
        contents: Vec<VerseContent>,
    },
    Br,
    Comment(String),
    Tag {
        #[serde(rename = "@name")]
        name: String,
        #[serde(rename = "$value", default)]
        contents: Vec<VerseContent>,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InstrumentLines {
    #[serde(rename = "$value", default)]
    pub contents: Vec<InstrumentContent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstrumentContent {
    Chord(InstrumentChord),
    Beat {
        #[serde(rename = "$value", default)]
        contents: Vec<InstrumentChord>,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InstrumentChord {
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@root", skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(rename = "@bass", skip_serializing_if = "Option::is_none")]
    pub bass: Option<String>,
    #[serde(rename = "@structure", skip_serializing_if = "Option::is_none")]
    pub structure: Option<String>,
    #[serde(rename = "@upbeat", skip_serializing_if = "Option::is_none")]
    pub upbeat: Option<bool>,
    #[serde(rename = "$value", default)]
    contents: Vec<InstrumentChord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn minimal() {
        let song: Song = from_str(
            r#"<song>
                <properties>
                    <titles>
                        <title>Title</title>
                    </titles>
                </properties>
                <lyrics>
                </lyrics>
            </song>"#,
        )
        .unwrap();

        assert_eq!(
            song,
            Song {
                properties: Properties {
                    titles: Titles {
                        titles: vec![Title {
                            lang: None,
                            translit: None,
                            original: None,
                            title: "Title".to_string(),
                        }]
                    },
                    authors: Authors { authors: vec![] },
                    copyright: None,
                    ccli_no: None,
                    released: None,
                    transposition: None,
                    tempo: None,
                    key: None,
                    time_signature: None,
                    variant: None,
                    publisher: None,
                    version: None,
                    keywords: None,
                    verse_order: None,
                    songbooks: Songbooks { songbooks: vec![] },
                    themes: Themes { themes: vec![] },
                    comments: Comments { comments: vec![] },
                },
                lyrics: Lyrics { lyrics: vec![] }
            }
        );
    }

    #[test]
    fn empty_lyrics() {
        let lyrics: Lyrics = from_str("<lyrics></lyrics>").unwrap();
        assert_eq!(lyrics, Lyrics { lyrics: vec![] });
    }

    #[test]
    fn verse_and_instrument() {
        let lyrics: Lyrics = from_str(
            r#"<lyrics>
                <instrument name="i"></instrument>
                <verse name="v1"></verse>
                <verse name="v2"></verse>
            </lyrics>"#,
        )
        .unwrap();
        assert_eq!(
            lyrics,
            Lyrics {
                lyrics: vec![
                    LyricEntry::Instrument {
                        name: "i".to_string(),
                        lines: vec![],
                    },
                    LyricEntry::Verse {
                        name: "v1".to_string(),
                        lang: None,
                        translit: None,
                        lines: vec![],
                    },
                    LyricEntry::Verse {
                        name: "v2".to_string(),
                        lang: None,
                        translit: None,
                        lines: vec![],
                    },
                ]
            }
        );
    }

    #[test]
    fn complex_verse() {
        let lyrics: Lyrics = from_str(
            r#"<lyrics>
                <verse name="v1" lang="en" translit="en">
                    <lines break="optional" part="men" repeat="2">
                        First line<br/>
                        <comment>Some comment</comment>
                        Second <chord root="D">line</chord>
                        <chord root="B"/>
                    </lines>
                    <lines>
                        More lines
                    </lines>
                </verse>
            </lyrics>"#,
        )
        .unwrap();
        assert_eq!(
            lyrics,
            Lyrics {
                lyrics: vec![LyricEntry::Verse {
                    name: "v1".to_string(),
                    lang: Some("en".to_string()),
                    translit: Some("en".to_string()),
                    lines: vec![
                        Lines {
                            break_optional: Some("optional".to_string()),
                            part: Some("men".to_string()),
                            repeat: Some(2),
                            contents: vec![
                                VerseContent::Text(
                                    "\n                        First line".to_string()
                                ),
                                VerseContent::Br,
                                VerseContent::Comment("Some comment".to_string()),
                                VerseContent::Text("\n                        Second ".to_string()),
                                VerseContent::Chord {
                                    root: Some("D".to_string()),
                                    name: None,
                                    bass: None,
                                    structure: None,
                                    upbeat: None,
                                    contents: vec![VerseContent::Text("line".to_string())],
                                },
                                VerseContent::Chord {
                                    root: Some("B".to_string()),
                                    name: None,
                                    bass: None,
                                    structure: None,
                                    upbeat: None,
                                    contents: vec![],
                                }
                            ],
                        },
                        Lines {
                            break_optional: None,
                            part: None,
                            repeat: None,
                            contents: vec![VerseContent::Text(
                                "\n                        More lines\n                    "
                                    .to_string()
                            )],
                        },
                    ],
                },]
            }
        );
    }

    #[test]
    fn instrument() {
        let lyrics: Lyrics = from_str(
            r#"<lyrics>
                <instrument name="i">
                    <lines>
                        <beat><chord root="A"/><chord root="B"/></beat>
                        <chord root="C"/>
                        <beat></beat>
                    </lines>
                </instrument>
            </lyrics>"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            lyrics,
            Lyrics {
                lyrics: vec![LyricEntry::Instrument {
                    name: "i".to_string(),
                    lines: vec![InstrumentLines {
                        contents: vec![
                            InstrumentContent::Beat {
                                contents: vec![
                                    InstrumentChord {
                                        root: Some("A".to_string()),
                                        ..Default::default()
                                    },
                                    InstrumentChord {
                                        root: Some("B".to_string()),
                                        ..Default::default()
                                    },
                                ]
                            },
                            InstrumentContent::Chord(InstrumentChord {
                                root: Some("C".to_string()),
                                ..Default::default()
                            }),
                            InstrumentContent::Beat { contents: vec![] },
                        ]
                    }],
                },]
            }
        );
    }

    #[test]
    fn instrument_nested_chords() {
        let lyrics: Lyrics = from_str(
            r#"<lyrics>
                <instrument name="i">
                    <lines>
                        <chord root="A"><chord root="B"/></chord>
                    </lines>
                </instrument>
            </lyrics>"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            lyrics,
            Lyrics {
                lyrics: vec![LyricEntry::Instrument {
                    name: "i".to_string(),
                    lines: vec![InstrumentLines {
                        contents: vec![InstrumentContent::Chord(InstrumentChord {
                            root: Some("A".to_string()),
                            contents: vec![InstrumentChord {
                                root: Some("B".to_string()),
                                ..Default::default()
                            }],
                            ..Default::default()
                        })]
                    }],
                },]
            }
        );
    }

    #[test]
    fn song_verses() {
        let song: Song = from_str(
            r#"<song>
                <properties>
                    <titles>
                        <title>Test</title>
                    </titles>
                </properties>
                <lyrics>
                    <verse name="v1"></verse>
                    <verse name="v2"></verse>
                </lyrics>
            </song>"#,
        )
        .unwrap();
        assert_eq!(
            song.lyrics,
            Lyrics {
                lyrics: vec![
                    LyricEntry::Verse {
                        name: "v1".to_string(),
                        lang: None,
                        translit: None,
                        lines: vec![],
                    },
                    LyricEntry::Verse {
                        name: "v2".to_string(),
                        lang: None,
                        translit: None,
                        lines: vec![],
                    },
                ]
            }
        );
    }

    #[test]
    fn serialise_empty() {
        let song = Song::default();
        assert_eq!(
            quick_xml::se::to_string(&song).unwrap(),
            "<song><properties><titles/></properties><lyrics/></song>"
        );
    }

    #[test]
    fn serialise_properties() {
        let song = Song {
            properties: Properties {
                titles: Titles {
                    titles: vec![Title {
                        title: "Title".to_string(),
                        lang: Some("en".to_string()),
                        ..Default::default()
                    }],
                },
                authors: Authors {
                    authors: vec![Author {
                        name: "Author".to_string(),
                        author_type: Some("words".to_string()),
                        ..Default::default()
                    }],
                },
                copyright: None,
                ccli_no: None,
                released: None,
                transposition: None,
                tempo: None,
                key: None,
                time_signature: None,
                variant: None,
                publisher: Some("Publisher".to_string()),
                version: None,
                keywords: None,
                verse_order: None,
                songbooks: Songbooks {
                    songbooks: vec![Songbook {
                        name: "Songbook".to_string(),
                        entry: Some("entry".to_string()),
                    }],
                },
                themes: Themes {
                    themes: vec![Theme {
                        title: "Theme".to_string(),
                        lang: Some("en".to_string()),
                        ..Default::default()
                    }],
                },
                comments: Comments {
                    comments: vec!["Comment".to_string(), "Another comment".to_string()],
                },
            },
            lyrics: Default::default(),
        };
        assert_eq!(
            quick_xml::se::to_string(&song).unwrap(),
            "<song>\
<properties>\
<titles><title lang=\"en\">Title</title></titles>\
<authors><author type=\"words\">Author</author></authors>\
<publisher>Publisher</publisher>\
<songbooks><songbook name=\"Songbook\" entry=\"entry\"/></songbooks>\
<themes><theme lang=\"en\">Theme</theme></themes>\
<comments><comment>Comment</comment><comment>Another comment</comment></comments>\
</properties>\
<lyrics/></song>"
        );
    }

    #[test]
    fn serialise_lyrics() {
        let song = Song {
            properties: Default::default(),
            lyrics: Lyrics {
                lyrics: vec![LyricEntry::Verse {
                    name: "v1".to_string(),
                    lang: Some("en".to_string()),
                    translit: None,
                    lines: vec![Lines {
                        part: Some("men".to_string()),
                        contents: vec![],
                        ..Default::default()
                    }],
                }],
            },
        };
        assert_eq!(
            quick_xml::se::to_string(&song).unwrap(),
            "<song>\
<properties><titles/></properties>\
<lyrics>\
<verse name=\"v1\" lang=\"en\">\
<lines part=\"men\"/>\
</verse>\
</lyrics>\
</song>"
        );
    }
}
