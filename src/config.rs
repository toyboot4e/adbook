use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `book.ron`
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub authors: Vec<String>,
    pub src: PathBuf,
    pub title: String,
}

pub struct Book {
    //
}
