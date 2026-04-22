use std::path::Path;

#[cfg(feature = "epub")]
use crate::parsers::epub::EpubParser;

use crate::parsers::{Parser, plain::PlainParser};

pub struct Chapter {
    pub content: String,
}
pub struct Book<const WIDTH: usize, const HEIGHT: usize> {
    chapters: Vec<Chapter>,
    chapter_index: usize,
    line_offset: usize,
    word_offset: usize,
}

impl<const WIDTH: usize, const HEIGHT: usize> Book<WIDTH, HEIGHT> {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let chapters = match path
            .extension()
            .expect("No file extension")
            .to_str()
            .expect("Not an OS string")
        {
            "txt" => Self::parse(PlainParser {}, path),
            #[cfg(feature = "epub")]
            "epub" => Self::parse(EpubParser {}, path),
            _ => panic!("Unknown file type"),
        };

        Ok(Self {
            chapters,
            chapter_index: 0,
            line_offset: 0,
            word_offset: 0,
        })
    }

    fn parse(parser: impl Parser<WIDTH, HEIGHT>, path: &Path) -> Vec<Chapter> {
        parser.parse(path)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Iterator for Book<WIDTH, HEIGHT> {
    type Item = [[u8; WIDTH]; HEIGHT];

    fn next(&mut self) -> Option<Self::Item> {
        let mut page = [[32; WIDTH]; HEIGHT];
        let mut y = 0;
        let mut x = 0;

        if self.line_offset
            >= self
                .chapters
                .iter()
                .map(|ch| ch.content.lines().count())
                .sum()
        {
            None
        } else {
            if let Some(chapter) = self.chapters.get(self.chapter_index) {
                'lines: for line in chapter.content.lines().skip(self.line_offset) {
                    for word in line.split(" ").skip(self.word_offset) {
                        if (WIDTH - x) < word.len() {
                            page[y][x..].iter_mut().for_each(|ch| *ch = b' ');
                            x = 0;
                            y += 1;
                            if y >= HEIGHT {
                                break 'lines;
                            }
                        }
                        for ch in word.chars().take(WIDTH) {
                            page[y][x] = ch as u8;
                            x += 1;
                        }
                        if x < WIDTH {
                            page[y][x] = b' ';
                            x += 1;
                        }
                        self.word_offset += 1;
                    }
                    self.word_offset = 0;
                    self.line_offset += 1;
                    x = 0;
                    y += 1;
                    if y >= HEIGHT {
                        break 'lines;
                    }
                }
                if self.line_offset >= chapter.content.lines().count() {
                    self.chapter_index += 1;
                    self.line_offset = 0;
                    self.word_offset = 0;
                }
                Some(page)
            } else {
                None
            }
        }
    }
}
