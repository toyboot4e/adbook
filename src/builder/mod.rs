//! Adbook directory builder

pub mod adoc;

use {
    anyhow::{Context, Result},
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

use crate::book::BookStructure;

pub trait BookBuilder {
    /// Walk through the [`BookStructure`] and build it into the site (destination) directory
    fn build_book_to_tmp_dir(
        &mut self,
        book: &BookStructure,
        cfg: &BuildConfig,
        out_dir: &Path,
    ) -> Result<()>;
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

    /// Where [`BookBuilder`] should output temporary files
    pub fn tmp_dir(&self) -> PathBuf {
        self.site.join(&self.tmp)
    }
}

/// Drives [`BookBuilder`] providing temporary output directory
pub fn run_builder(
    builder: &mut impl BookBuilder,
    book: &BookStructure,
    cfg: &BuildConfig,
) -> Result<()> {
    let out_dir = cfg.tmp_dir();

    // make sure we have an available temporary output directory
    if out_dir.exists() {
        ensure!(
            out_dir.is_dir(),
            "There's something that prevents `adbook` from making a temporary output directory: {}",
            out_dir.display()
        );

        // can we use it as a temporary output directory?
        println!("-----------------------------------------------------------");
        println!("There's already a directory where `adbook` wants to output temporary files:");
        println!("{}", out_dir.display());
        println!("Is it OK to clear all files in that directory and use it as a temporary output directory?");

        match rprompt::prompt_reply_stdout("> [y/n]: ")?.as_str() {
            "y" | "yes" => {}
            _ => {
                println!("Stopped building adbook directory");
                return Ok(());
            }
        }

        trace!(
            "Creating the temporary output directory at: {}",
            out_dir.display()
        );

        fs::remove_dir_all(&out_dir).with_context(|| format!(
                "Unexpected error while clearing an output directory so that `adbook` can use it: {}",
                out_dir.display()
            ))?;
    }

    assert!(
            !out_dir.exists(),
            "Fatal error: adbook must have ensured that temporary output directory doesn't exist at: {}",
            out_dir.display()
        );

    // now create the temporary outputting directory
    fs::create_dir(&out_dir).with_context(|| {
        format!(
            "Failed to temporary output directory at: {}",
            out_dir.display()
        )
    })?;

    trace!(
        "Created a new temporary output directly at: {}",
        out_dir.display()
    );

    builder.build_book_to_tmp_dir(book, cfg, &out_dir)?;
    // TODO: clear the destination and copy the temporary outputs to the actual destination

    trace!("==> Removing the temporary output directory");
    fs::remove_dir_all(&out_dir).with_context(|| {
        format!(
            "Unexpected error when removing the temporary output directory at: {}",
            out_dir.display()
        )
    })?;

    Ok(())
}
