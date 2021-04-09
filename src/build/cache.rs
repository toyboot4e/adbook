/*!
Skip running `asciidoctor` if a file is not modofied since the last run
*/

use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::*;
use serde::{Deserialize, Serialize};

use crate::book::BookStructure;

pub fn clear_cache(book: &BookStructure) -> io::Result<()> {
    let root = CacheIndex::locate_cache_root(book);
    if !root.is_dir() {
        return Ok(());
    }
    fs::remove_dir_all(root)?;
    Ok(())
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

impl Default for CacheData {
    fn default() -> Self {
        Self { entries: vec![] }
    }
}

impl CacheData {
    pub fn empty() -> Self {
        Self { entries: vec![] }
    }

    /// Create s cache from the source directory of a book
    pub fn create_new_cache(book: &BookStructure) -> Result<Self> {
        let src_dir = book.src_dir_path();
        let mut entries = Vec::new();
        crate::utils::visit_files_rec(&src_dir, &mut |src_file| {
            let rel_path = src_file.strip_prefix(&src_dir).unwrap();
            let last_modified = {
                let metadata = fs::metadata(src_file)?;
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
    old: Option<CacheData>,
    new: CacheData,
}

impl CacheDiff {
    fn create(book: &BookStructure, old_cache: Option<CacheData>) -> Result<Self> {
        let now = CacheData::create_new_cache(book)?;
        Ok(Self {
            old: old_cache,
            new: now,
        })
    }

    pub fn into_new_cache_data(self) -> CacheData {
        self.new
    }

    /// If the file needs to be rebuilt
    ///
    /// * `src_path`: Either absolute path or relative path from the source directory
    pub fn need_build(&self, book: &BookStructure, src_path: &Path) -> bool {
        let rel_path = if src_path.is_absolute() {
            &src_path.strip_prefix(book.src_dir_path()).unwrap()
        } else {
            src_path
        };

        let current_entry = self
            .new
            .find_cache(rel_path)
            .unwrap_or_else(|| panic!("given non-existing file in source directory"));

        let last_entry = {
            let last = match self.old.as_ref() {
                Some(cache) => cache,
                None => return true,
            };

            match last.find_cache(rel_path) {
                Some(cache) => cache,
                None => return true,
            }
        };

        last_entry.last_modified != current_entry.last_modified
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct CacheIndex {
    cache: CacheData,
}

impl CacheIndex {
    fn locate_cache_root(book: &BookStructure) -> PathBuf {
        let cache_dir = book.root.join(".adbook-cache/");
        crate::utils::validate_dir(&cache_dir).expect("Unable to locate cache directory");
        cache_dir
    }

    fn locate_index(book: &BookStructure) -> PathBuf {
        let cache_dir = book.root.join(".adbook-cache/");
        crate::utils::validate_dir(&cache_dir).expect("Unable to locate cache directory");
        cache_dir.join("index")
    }

    pub fn empty() -> Self {
        Self {
            cache: CacheData::empty(),
        }
    }

    pub fn load(book: &BookStructure) -> Result<Self> {
        let index = Self::locate_index(book);
        if !index.is_file() {
            Ok(Default::default())
        } else {
            let me = ron::de::from_reader(File::open(&index)?)?;
            Ok(me)
        }
    }

    pub fn create_diff(&self, book: &BookStructure) -> Result<CacheDiff> {
        if self.cache.entries.is_empty() {
            CacheDiff::create(book, None)
        } else {
            CacheDiff::create(book, Some(self.cache.clone()))
        }
    }

    /// `.cache_dir/a`; old files will be here
    pub fn locate_old_cache_dir(book: &BookStructure) -> Result<PathBuf> {
        let cache_dir = Self::locate_cache_root(book);
        let tmp_dir = cache_dir.join("a");
        crate::utils::validate_dir(&tmp_dir)?;
        Ok(tmp_dir)
    }

    /// `.cache_dir/b`; create new files here
    pub fn locate_new_cache_dir(book: &BookStructure) -> Result<PathBuf> {
        let cache_dir = Self::locate_cache_root(book);
        let tmp_dir = cache_dir.join("b");
        crate::utils::validate_dir(&tmp_dir)?;
        Ok(tmp_dir)
    }

    /// Cleans up the temporary directory and saves build cache
    pub fn clean_up_and_save(&self, book: &BookStructure, new_cache: CacheData) -> Result<()> {
        // copy htlm files
        let old = Self::locate_old_cache_dir(book)?;
        let new = Self::locate_new_cache_dir(book)?;
        // rm -rf old
        fs::remove_dir_all(&old)?;
        // mv new old
        fs::rename(new, &old)?;

        // save cacke
        let index = Self::locate_index(book);
        let ron = ron::ser::to_string(&Self { cache: new_cache })?;
        fs::write(&index, ron.as_bytes())?;
        Ok(())
    }
}
