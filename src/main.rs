use openlyrics::Song;
use quick_xml::{de::from_reader, se::to_string};
use std::io::stdin;

fn main() {
    let song: Song = from_reader(stdin().lock()).unwrap();
    println!("{song:#?}");
    let xml = to_string(&song).unwrap();
    println!("{xml}");
}
