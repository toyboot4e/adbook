/*!
Skip running `asciidoctor` if a file is not modofied since the last run

TODO: rebuild the whole project when the number of source files or article title changes.

# Cache directory

```
.adbook-cache
├── a               # cached html files
│   ├── 404.html
│   └── index.html
└── index           # cache index
```
*/

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::*;
use serde::{Deserialize, Serialize};

use crate::book::BookStructure;

pub fn clear_cache(book: &BookStructure) -> io::Result<()> {
    let root = CacheIndex::locate_root(book);
    if !root.is_dir() {
        return io::Result::Ok(());
    }
    fs::remove_dir_all(root)?;
    io::Result::Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CacheIndexData {
    entries: Vec<CacheIndexEntry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CacheIndexEntry {
    last_modified: SystemTime,
    /// Relative path from source directory
    path: PathBuf,
}

impl Default for CacheIndexData {
    fn default() -> Self {
        Self { entries: vec![] }
    }
}

impl CacheIndexData {
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
            entries.push(CacheIndexEntry {
                last_modified,
                path: rel_path.to_path_buf(),
            });
            Ok(())
        })?;
        Ok(Self { entries })
    }

    pub fn find_cache(&self, rel_path: &Path) -> Option<&CacheIndexEntry> {
        for e in &self.entries {
            if e.path == rel_path {
                return Some(e);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct CacheIndexDiff {
    old: Option<CacheIndexData>,
    new: CacheIndexData,
}

impl CacheIndexDiff {
    fn create(book: &BookStructure, old_cache: Option<CacheIndexData>) -> Result<Self> {
        let now = CacheIndexData::create_new_cache(book)?;
        Ok(Self {
            old: old_cache,
            new: now,
        })
    }

    pub fn into_new_cache_data(self) -> CacheIndexData {
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
    cache: CacheIndexData,
}

impl CacheIndex {
    fn locate_root(book: &BookStructure) -> PathBuf {
        let root_dir = book.root.join(".adbook-cache/");
        crate::utils::validate_dir(&root_dir).expect("Unable to locate cache directory");
        root_dir
    }

    fn locate_index(book: &BookStructure) -> PathBuf {
        let root_dir = book.root.join(".adbook-cache/");
        crate::utils::validate_dir(&root_dir).expect("Unable to locate cache directory");
        root_dir.join("index")
    }

    pub fn empty() -> Self {
        Self {
            cache: CacheIndexData::empty(),
        }
    }

    pub fn load(book: &BookStructure) -> Result<Self> {
        let index = Self::locate_index(book);
        if !index.is_file() {
            Ok(Default::default())
        } else {
            let s = fs::read(&index)?;
            let me = bincode::deserialize(&s).with_context(|| {
                anyhow!("Error on deserializing cache. Try `adbook clear` if you update `adbook`.")
            })?;
            Ok(me)
        }
    }

    pub fn create_diff(&self, book: &BookStructure) -> Result<CacheIndexDiff> {
        if self.cache.entries.is_empty() {
            CacheIndexDiff::create(book, None)
        } else {
            CacheIndexDiff::create(book, Some(self.cache.clone()))
        }
    }

    /// `.cache_dir/a`; old files will be here
    pub fn locate_cache_dir(book: &BookStructure) -> Result<PathBuf> {
        let root_dir = Self::locate_root(book);
        let cache_dir = root_dir.join("a");
        crate::utils::validate_dir(&cache_dir)?;
        Ok(cache_dir)
    }

    /// Cleans up the temporary output directory and saves build cache
    pub fn update_cache_index(
        &self,
        book: &BookStructure,
        new_cache: CacheIndexData,
    ) -> Result<()> {
        // save index
        let index = Self::locate_index(book);
        let bin = bincode::serialize(&Self { cache: new_cache })?;
        fs::write(&index, bin)?;
        Ok(())
    }
}
