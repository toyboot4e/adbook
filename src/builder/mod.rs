//! Adbook directory builder

pub mod adoc;

use {
    anyhow::Result,
    std::path::{Path, PathBuf},
};

use crate::book::BookStructure;

pub trait BookBuilder {
    /// Walk through the [`BookStructure`] and build it into the site (destination) directory
    fn build_book(&mut self, book: &BookStructure, cfg: &BuildConfig) -> Result<()>;
}

pub struct BuildConfig {
    /// Absolute path to site directory
    site: PathBuf,
    /// Path to directory where a renderer outputs temporary files
    tmp: PathBuf,
}

impl BuildConfig {
    pub fn from_site_path(site: &Path) -> Self {
        Self {
            site: site.to_owned(),
            tmp: "_tmp_".into(),
        }
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.site.join(&self.tmp)
    }
}
