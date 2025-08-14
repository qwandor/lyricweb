use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Song {
    pub properties: Properties,
    pub lyrics: Lyrics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub titles: Titles,
    #[serde(default, skip_serializing_if = "Authors::is_empty")]
    pub authors: Authors,
    pub copyright: Option<String>,
    pub ccli_no: Option<u64>,
    pub released: Option<String>,
    pub transposition: Option<i8>,
    pub tempo: Option<Tempo>,
    pub key: Option<String>,
    pub time_signature: Option<String>,
    pub variant: Option<String>,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub keywords: Option<String>,
    // TODO: Parse space-separated values into a Vec.
    pub verse_order: Option<String>,
    #[serde(default, skip_serializing_if = "Songbooks::is_empty")]
    pub songbooks: Songbooks,
    #[serde(default, skip_serializing_if = "Themes::is_empty")]
    pub themes: Themes,
    #[serde(default, skip_serializing_if = "Comments::is_empty")]
    pub comments: Comments,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Titles {
    #[serde(rename = "title")]
    pub titles: Vec<Title>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Title {
    #[serde(rename = "@lang")]
    pub lang: Option<String>,
    #[serde(rename = "@translit")]
    pub translit: Option<String>,
    #[serde(rename = "@original")]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Author {
    #[serde(rename = "@type")]
    pub author_type: Option<String>,
    #[serde(rename = "@lang")]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Songbook {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@entry")]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Theme {
    #[serde(rename = "@lang")]
    pub lang: Option<String>,
    #[serde(rename = "@translit")]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lyrics {}
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
                lyrics: Lyrics {}
            }
        );
    }
}
