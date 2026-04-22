use rbook::{
    epub::{self, reader::EpubReaderSettings},
    prelude::*,
};
use std::path::Path;
use xhtml_parser::{Document, defs::NodeIdx};

use crate::{book::Chapter, parsers::Parser};

pub struct EpubParser;
impl EpubParser {
    fn parse_spine(&self, path: &Path, width: usize, height: usize) -> Vec<Chapter> {
        let mut parsed = Vec::new();

        let epub = epub::Epub::open(path).expect("Could not open");
        let spine = epub.spine();
        let mut chapters = Vec::new();
        for entry in spine {
            if let Some(resource) = entry.resource()
                && *resource.kind() == "application/xhtml+xml".into()
            {
                let mut ignore = Vec::new();
                let mut current = 0;
                let mut offset = 0;
                let bytes = epub.read_resource_bytes(resource.key()).unwrap();
                let doc = Document::new(bytes).expect("Could not parse xml document");
                while let Some(node) = doc.next_seq_node(current) {
                    match node.node_info.node_type() {
                        // xhtml_parser::NodeType::Element { name, attributes } => {
                        //     match doc.get_str_from_location(name.clone()) {
                        //         "h1" | "h2" | "h3" | "p" => {
                        //             let (content, count) = get_text_recursive(&doc, node.idx, 0);
                        //             parsed.push(content.join(""));
                        //             offset = count;
                        //         }
                        //         _ => {}
                        //     }
                        // }
                        xhtml_parser::NodeType::Element { name, attributes } => {
                            match doc.get_str_from_location(name.clone()) {
                                "h1" | "h2" | "h3" | "p" => {
                                    if !parsed.is_empty() {
                                        parsed.push(String::from("\n"));
                                    }
                                }
                                "img" => {
                                    parsed.push(
                                        format!(
                                            "\n[{}]\n",
                                            node.attributes()
                                                .find(|a| a.name() == "alt")
                                                .map_or("", |a| a.value())
                                        )
                                        .to_string(),
                                    );
                                }
                                "head" => {
                                    node.children().for_each(|n| ignore.push(n.idx));
                                }
                                _ => {}
                            }
                        }
                        xhtml_parser::NodeType::Text(loc) => {
                            if let Some(parent) = node.parent()
                                && !ignore.contains(&parent.idx)
                            {
                                parsed.push(doc.get_str_from_location(loc.clone()).to_string())
                            }
                        }
                        xhtml_parser::NodeType::Head => {
                            println!("{:?}", node.node_info);
                            ignore.push(node.idx);
                            if let Some(sib) = node.parent() {
                                current = sib.idx;
                                continue;
                            } else {
                                continue;
                            }
                        }
                    }
                    current = node.idx + offset;
                    offset = 0;
                }
            }
            chapters.push(Chapter {
                content: parsed.join(""),
            });
            parsed.clear();
        }
        chapters
    }
}

fn get_text_recursive(doc: &Document, idx: NodeIdx, acc: u16) -> (Vec<String>, u16) {
    let node = doc.get_node(idx).expect("No such node");
    let mut content = Vec::new();
    let mut desc_len = 0;

    for desc in node.children() {
        let (desc_vec, len) = get_text_recursive(doc, desc.idx, acc + 1);
        for el in desc_vec {
            content.push(el);
        }
        desc_len = len;
    }

    if let Some(node_text) = node.text() {
        match node.tag_name() {
            "h1" | "h2" | "h3" | "p" => {
                println!("new par");
                content.push(String::from("\n"));
            }
            _ => {}
        }
        content.push(
            node_text
                .split("\n")
                .map(|s| {
                    s.chars()
                        .map(|c| {
                            if !c.is_ascii() || c < ' ' {
                                match c {
                                    '“' | '”' => '"',
                                    '’' => '\'',
                                    '—' => '-',
                                    _ => ' ',
                                }
                            } else {
                                c
                            }
                        })
                        .collect::<String>()
                        .replace("\n", " ")
                        .replace("\t", "")
                        .replace("\r", "")
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
                .join("_"),
        );
    } else {
        match node.tag_name() {
            "br" => content.push(String::from("\n")),
            _ => {}
        }
    }

    (content, desc_len)
}

impl<const WIDTH: usize, const HEIGHT: usize> Parser<WIDTH, HEIGHT> for EpubParser {
    fn parse(&self, path: &Path) -> Vec<Chapter> {
        self.parse_spine(path, WIDTH, HEIGHT)
        // let mut buf = Vec::new();
        // let epub = epub::Epub::open(path).expect("Could not open");
        // let mut reader = epub.reader_with(
        //     EpubReaderSettings::builder()
        //         .linear_behavior(epub::reader::LinearBehavior::Original)
        //         .build(),
        // );
        // while let Some(Ok(content)) = reader.read_next() {
        //     buf.push(content.into_string())
        // }
        // let joined = buf.join("");
        // let doc = Document::new(joined.as_bytes().to_vec()).expect("Could not parse xml document");

        // let mut current = 0;
        // let mut parsed = Vec::new();
        // let mut offset = 0;
        // while let Some(node) = doc.next_seq_node(current) {
        //     match node.node_info.node_type() {
        //         xhtml_parser::NodeType::Element { name, attributes } => {
        //             match doc.get_str_from_location(name.clone()) {
        //                 "h1" | "h2" | "h3" | "p" => {
        //                     let (content, count) = get_text_recursive(&doc, node.idx, 0);
        //                     parsed.push(content.join(""));
        //                     offset = count;
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         _ => {}
        //     }
        //     current = node.idx + offset;
        //     offset = 0;
        // }
        // parsed.join("\n")
    }
}
