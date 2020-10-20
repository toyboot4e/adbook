//! Book builder

mod builtin;

use {
    anyhow::{Context, Error, Result},
    std::{fs, io, path::Path},
};

use crate::book::{config::CmdOptions, BookStructure};

/// Builds an `adbook` project with a configuration
pub fn build_book(book: &BookStructure) -> Result<()> {
    let mut builder = self::builtin::BuiltinBookBuilder::new();
    self::run_builder(&mut builder, book)
}

/// Converts an AsciiDoc file into html using some configuration
pub fn convert_adoc(src: &Path, dst: &Path) -> Result<String> {
    ensure!(src.is_file(), "Given invalid source file path");
    ensure!(!dst.exists(), "Something in the destination file path");

    // setup dummy context & builder for an article
    use crate::builder::builtin::{BuildContext, BuiltinBookBuilder};
    let opts: CmdOptions = vec![("--embedded".to_string(), vec![])];
    let mut bcx = BuildContext::single_article(src, dst, opts)?;
    let mut builder = BuiltinBookBuilder::new();

    let mut buf = String::with_capacity(5 * 1024);
    builder.run_asciidoctor_to_buf(src, dst, &mut buf, &mut bcx)?;

    Ok(buf)
}

// --------------------------------------------------------------------------------
// Structure for implementing custom builder (hidden for now)

/// adbook builder run via [`run_builder`]
trait BookBuilder {
    /// Walk through the [`BookStructure`] and build it into the temporary directory
    ///
    /// Creation & deletion of the temporay output directory is done via [`run_builder`].
    fn build_book_to_tmp_dir(&mut self, book: &BookStructure, out_dir: &Path) -> Result<()>;
}

/// Runs [`BookBuilder`] with guards for temporary directory
fn run_builder(builder: &mut impl BookBuilder, book: &BookStructure) -> Result<()> {
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
    let out_dir = book.site_dir_path().join("__tmp__");
    if !self::validate_out_dir(&out_dir)? {
        println!("Stopped building adbook directory");
        return Ok(());
    }

    // now let's build the project!
    builder.build_book_to_tmp_dir(book, &out_dir)?;

    trace!("==> Copying output files to site directory");
    {
        let mut errors = Vec::with_capacity(10);
        let res = self::copy_outputs(book, &out_dir, &mut errors);
        crate::utils::print_errors(&errors, "while copying temporary files to site directory");
        res?;
    }

    trace!(
        "==> Removing the temporary output directory: {}",
        out_dir.display()
    );

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

fn copy_outputs(book: &BookStructure, out_dir: &Path, errors: &mut Vec<Error>) -> Result<()> {
    // clear most files in site directory
    let site_dir = book.site_dir_path();
    for entry in fs::read_dir(&site_dir).context("Unable to `read_dir` the source directory")? {
        let entry = entry.context("Unable to read some entry when reading source directory")?;

        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or(anyhow!("Unable to turn OsStr to &str (`copy_outputs`)"))?;

        // filter `.git` and temporary output directory
        if name.starts_with(".") || site_dir.join(name) == out_dir {
            continue;
        }

        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            fs::create_dir_all(&path)?;
        } else {
            // TODO: how can we remove symlinks?
            errors.push(anyhow!(
                "Unexpected file when clearing site directroy: {}",
                path.display()
            ));
        }
    }

    // copy the `includes` files in `book.ron`
    for rel_path in &book.book_ron.includes {
        // ensure the given path is valid
        if !rel_path.is_relative() {
            errors.push(anyhow!(
                "Non-relative path in `book.ron` includes: {}",
                rel_path.display()
            ));
            continue;
        }

        let src_path = book.src_dir_path().join(rel_path);
        let dst_path = book.site_dir_path().join(rel_path);

        // ensure the source file/directory exists
        if !src_path.exists() {
            errors.push(anyhow!(
                "Not a valid relative path from the source directroy in `book.ron` includes: {}",
                rel_path.display()
            ));
            continue;
        }

        // let's copy
        if src_path.is_file() {
            // case 1. file
            let dir = src_path.parent().unwrap();

            // create parent directory
            if !dir.exists() {
                fs::create_dir_all(dir).with_context(|| {
                    format!(
                        "Unable to create parent directory of included file: {}",
                        src_path.display(),
                    )
                })?;
            }

            fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "Unable to copy source included file `{}` to `{}`",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        } else if src_path.is_dir() {
            // case 2. directory
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path).with_context(|| {
                    format!(
                        "Unable to create parent directory:\nsrc: {}\ndst: {}",
                        src_path.display(),
                        dst_path.display(),
                    )
                })?;
            }

            crate::utils::copy_items_rec(&src_path, &dst_path).with_context(|| {
                format!(
                    "Unable to copy included directory:\nsrc: {}\ndst: {}",
                    src_path.display(),
                    dst_path.display(),
                )
            })?;
        } else {
            // case 3. unexpected kind of file
            errors.push(anyhow!(
                "Unexpected kind of file to include in `book.ron`: {}",
                src_path.display()
            ));
        }
    }

    // copy the output files to site directory
    for entry in fs::read_dir(&out_dir).context("Unable to `read_dir` the tmp directory")? {
        let entry = entry.context("Unable to read some entry when reading tmp directory")?;

        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(out_dir).unwrap();
        let dst_path = site_dir.join(rel_path);

        if src_path.is_file() {
            trace!("* `{}` -> `{}`", src_path.display(), dst_path.display());
            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            crate::utils::copy_items_rec(&src_path, &dst_path)?;
        } else {
            errors.push(anyhow!(
                "Unexpected kind of file in temporary output directory: `{}`",
                src_path.display(),
            ));
        }
    }

    Ok(())
}
