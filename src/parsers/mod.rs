use std::path::Path;

use crate::book::Chapter;

#[cfg(feature = "epub")]
pub mod epub;

pub mod plain;

pub trait Parser<const WIDTH: usize, const HEIGHT: usize> {
    fn parse(&self, path: &Path) -> Vec<Chapter>;
}
