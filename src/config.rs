use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `book.ron`
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub authors: Vec<String>,
    pub title: String,
    pub src: PathBuf,
    pub out: PathBuf,
}

pub struct Book {
    //
}
