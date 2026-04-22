use std::{fs::File, io::Read, path::Path};

use crate::{book::Chapter, parsers::Parser};

pub struct PlainParser;

impl<const WIDTH: usize, const HEIGHT: usize> Parser<WIDTH, HEIGHT> for PlainParser {
    fn parse(&self, path: &Path) -> Vec<Chapter> {
        let mut buf = String::new();
        let mut file = File::open(path).expect("Could not open the file");
        let count = file
            .read_to_string(&mut buf)
            .expect("Not valid UTF-8 content");
        println!("Parsing {} bytes...", count);
        vec![Chapter { content: buf }]
    }
}
