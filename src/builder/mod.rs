//! Book builder

mod builtin;

use {
    anyhow::{Context, Result},
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

use crate::book::BookStructure;

use self::builtin::BuiltinBookBuilder;

/// Builds an `adbook` project with a configuration
pub fn build(book: &BookStructure, cfg: &BuildConfig) -> Result<()> {
    let mut builder = BuiltinBookBuilder::new();
    self::run_builder(&mut builder, book, cfg)
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Absolute path to site directory
    site: PathBuf,
    /// Path to directory where a renderer outputs temporary files
    tmp: PathBuf,
}

impl BuildConfig {
    /// Creates default configuration from path to a destination (site) directory
    pub fn from_site_path(site: &Path) -> Self {
        Self {
            site: site.to_owned(),
            tmp: "_tmp_".into(),
        }
    }

    /// adbook builder should output temporary files to this directory
    pub fn tmp_dir(&self) -> PathBuf {
        self.site.join(&self.tmp)
    }
}

// --------------------------------------------------------------------------------
// Structure for implementing custom builder (hidden for now)

/// adbook builder run via [`run_builder`]
trait BookBuilder {
    /// Walk through the [`BookStructure`] and build it into the temporary directory
    ///
    /// Creation & deletion of the temporay output directory is done via [`run_builder`].
    fn build_book_to_tmp_dir(
        &mut self,
        book: &BookStructure,
        cfg: &BuildConfig,
        out_dir: &Path,
    ) -> Result<()>;
}

/// Runs [`BookBuilder`] with guards for temporary directory
fn run_builder(
    builder: &mut impl BookBuilder,
    book: &BookStructure,
    cfg: &BuildConfig,
) -> Result<()> {
    // create site (destination) directory if there's not
    {
        let site = book.site_dir_path();
        if !site.exists() {
            fs::create_dir(&site).with_context(|| {
                format!("Failed to create site directory at: {}", site.display())
            })?;
        }
    }

    // create temporary directory if there's not
    let out_dir = cfg.tmp_dir();
    if !self::validate_out_dir(&out_dir)? {
        println!("Stopped building adbook directory");
        return Ok(());
    }

    // now let's build the project!
    builder.build_book_to_tmp_dir(book, cfg, &out_dir)?;

    // TODO: clear files in site directory one by one except `.git` etc.

    // copy the output files and includes to the site
    for rel_path in &book.book_ron.includes {
        let path = book.root.join(rel_path);
        if path.is_file() {
            //
        }
        if path.is_dir() {
            //
        }
    }

    // trace!("==> Removing the temporary output directory");
    fs::remove_dir_all(&out_dir).with_context(|| {
        format!(
            "Unexpected error when removing the temporary output directory at: {}",
            out_dir.display()
        )
    })?;

    Ok(())
}

/// Returns if the directory is valid or not
fn validate_out_dir(out_dir: &Path) -> Result<bool> {
    // make sure we have an available temporary output directory
    if out_dir.exists() {
        ensure!(
            out_dir.is_dir(),
            "There's something that prevents `adbook` from making a temporary output directory at: {}",
            out_dir.display()
        );

        // ask user if `adbook` is allowed to clear the to-be temporary directory
        println!("-----------------------------------------------------------");
        println!("There's already a directory where `adbook` wants to output temporary files:");
        println!("{}", out_dir.display());
        println!("Is it OK to clear all files in that directory and use it as a temporary output directory?");

        match rprompt::prompt_reply_stdout("> [y/n]: ")?.as_str() {
            "y" | "yes" => {}
            _ => {
                return Ok(false);
            }
        }

        trace!(
            "Creating the temporary output directory at: {}",
            out_dir.display()
        );

        fs::remove_dir_all(&out_dir).with_context(|| {
            format!(
                "Unexpected error while clearing an output directory for `adbook`: {}",
                out_dir.display()
            )
        })?;
    }

    // now `out_dir` must NOT exist
    assert!(
        !out_dir.exists(),
        "Fatal error: adbook must have ensured that temporary output directory doesn't exist at: {}",
        out_dir.display()
    );

    // create the temporary outputting directory
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

    Ok(true)
}
