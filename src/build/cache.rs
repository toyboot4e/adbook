/*!
Skip running `asciidoctor` if a file is not modofied since the last run
*/

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::*;
use serde::{Deserialize, Serialize};

use crate::book::BookStructure;

fn cache_path(book: &BookStructure) -> PathBuf {
    book.root.join(".adbook-cache")
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CacheEntry {
    last_modified: SystemTime,
    /// Relative path from source directory
    path: PathBuf,
}

/// Timestamps stored at `<root>/.adbook-cache`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CacheData {
    entries: Vec<CacheEntry>,
}

impl CacheData {
    pub fn load(book: &BookStructure) -> Result<Option<Self>> {
        let cache_file_path = self::cache_path(book);

        if !cache_file_path.exists() {
            Ok(None)
        } else if cache_file_path.is_file() {
            // load
            let cache = ron::de::from_reader(File::open(cache_file_path)?)
                .with_context(|| "cannot deserialize adbook cache file")?;
            Ok(Some(cache))
        } else {
            bail!("cannot create adbook cache file");
        }
    }

    /// Create s cache from the source directory of a book
    pub fn create(book: &BookStructure) -> Result<CacheData> {
        let src_dir = book.src_dir_path();
        let mut entries = Vec::new();
        crate::utils::visit_files_rec(&src_dir, &mut |f| {
            let rel_path = src_dir.strip_prefix(&src_dir).unwrap();
            let last_modified = {
                let metadata = fs::metadata(f)?;
                metadata.modified()?
            };
            entries.push(CacheEntry {
                last_modified,
                path: rel_path.to_path_buf(),
            });
            Ok(())
        })?;
        Ok(CacheData { entries })
    }

    pub fn write(&self, book: &BookStructure) -> Result<()> {
        let s = ron::ser::to_string(self)?;
        let path = self::cache_path(book);
        fs::write(path, s)?;
        Ok(())
    }

    pub fn find_cache(&self, rel_path: &Path) -> Option<&CacheEntry> {
        for e in &self.entries {
            if e.path == rel_path {
                return Some(e);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct CacheDiff {
    last: Option<CacheData>,
    now: CacheData,
}

impl CacheDiff {
    pub fn load(book: &BookStructure) -> Result<Self> {
        let last = CacheData::load(book)?;
        let now = CacheData::create(book)?;
        Ok(Self { last, now })
    }

    /// If the file needs to be rebuilt
    pub fn need_build(&self, book: &BookStructure, path: &Path) -> bool {
        let rel_path = if path.is_absolute() {
            &path.strip_prefix(book.src_dir_path()).unwrap()
        } else {
            path
        };

        let current_entry = match self.now.find_cache(rel_path) {
            Some(cache) => cache,
            None => panic!("Given non-existing file path in cache"),
        };

        let last_entry = {
            let last = match self.last.as_ref() {
                Some(cache) => cache,
                None => return false,
            };

            match last.find_cache(rel_path) {
                Some(cache) => cache,
                None => return true,
            }
        };

        last_entry.last_modified != current_entry.last_modified
    }
}
