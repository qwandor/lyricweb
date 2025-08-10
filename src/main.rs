use openlyrics::Song;
use serde_xml_rs::{from_reader, to_string};
use std::io::stdin;

fn main() {
    let song: Song = from_reader(stdin()).unwrap();
    println!("{song:#?}");
    let xml = to_string(&song).unwrap();
    println!("{xml}");
}
