# openlyrics

[![crates.io page](https://img.shields.io/crates/v/openlyrics.svg)](https://crates.io/crates/openlyrics)
[![docs.rs page](https://docs.rs/openlyrics/badge.svg)](https://docs.rs/openlyrics)

A Rust library for parsing [OpenLyrics](https://docs.openlyrics.org/en/latest/) XML files.

This is not an officially supported Google product. This project is not eligible for the
[Google Open Source Software Vulnerability Rewards Program](https://bughunters.google.com/open-source-security).

## Usage

```rust
use openlyrics::types::Song;
use quick_xml::de::from_reader;
use std::{fs::File, io::BufReader};

let song = from_reader::<_, Song>(BufReader::new(File::open("song.xml")?))?;
println!("Title: {}", song.properties.titles.titles[0].title);
```

## License

Licensed under either of

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

If you want to contribute to the project, see details of
[how we accept contributions](../CONTRIBUTING.md).
