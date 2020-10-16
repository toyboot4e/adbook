//! Builtin adbook renderer for AsciiDoc

use {
    anyhow::{Context, Result},
    std::fs::{self},
};

use crate::{
    book::BookStructure,
    builder::{BookBuilder, BuildConfig},
};

pub struct AdocBuilder {}

impl AdocBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl BookBuilder for AdocBuilder {
    fn build_book(&mut self, book: &BookStructure, cfg: &BuildConfig) -> Result<()> {
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

        // build the directory
        // copy the temporary outputs to the actual destination

        trace!("==> Removing the temporary output directory");
        fs::remove_dir_all(&out_dir).with_context(|| {
            format!(
                "Unexpected error when removing the temporary output directory at: {}",
                out_dir.display()
            )
        })?;

        Ok(())
    }
}
