#![allow(unused)]

pub struct Info {
    title: String,
    author: String,
    publish_date: String,
}

pub struct Chapter {
    title: String,
    content: String,
}

pub struct BookerBook {
    info: Info,
    chapters: Vec<Chapter>,
}
