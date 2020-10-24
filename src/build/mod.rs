//! Book builder

pub mod convert;
pub mod visit;

use {
    anyhow::{Context, Error, Result},
    std::{fs, path::Path},
};

use crate::{
    book::{
        walk::{walk_book, BookVisitor},
        BookStructure,
    },
    build::visit::AdocBookVisitor,
};

/// Builds an `adbook` structure into a site directory
///
/// Actually, into a temporary directory and then a site directory.
pub fn build_book(book: &BookStructure) -> Result<()> {
    let mut builder = AdocBookVisitor::new(book.book_ron.adoc_opts.clone());
    self::build_book_impl(&mut builder, book)
}

/// book -> tmp -> site
fn build_book_impl(v: &mut impl BookVisitor, book: &BookStructure) -> Result<()> {
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
    let tmp_dir = book.site_dir_path().join("__tmp__");
    if !self::validate_out_dir(&tmp_dir)? {
        println!("Stopped building adbook directory");
        return Ok(());
    }

    // now let's build the project!
    walk_book(v, &book.toc, &book.src_dir_path(), &tmp_dir)?;

    info!("===> Copying output files to site directory");
    {
        let mut errors = Vec::with_capacity(10);
        let res = self::overwrite_site_with_temporary_outputs(book, &tmp_dir, &mut errors);
        crate::utils::print_errors(&errors, "while copying temporary files to site directory");
        res?;
    }

    info!(
        "===> Removing the temporary output directory: {}",
        tmp_dir.display()
    );

    fs::remove_dir_all(&tmp_dir).with_context(|| {
        format!(
            "Unexpected error when removing the temporary output directory at: {}",
            tmp_dir.display()
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

/// TODO: refactor
fn overwrite_site_with_temporary_outputs(
    book: &BookStructure,
    out_dir: &Path,
    errors: &mut Vec<Error>,
) -> Result<()> {
    let site_dir = book.site_dir_path();

    // clear most files in site directory
    trace!("remove files in site directory");
    for entry in fs::read_dir(&site_dir).context("Unable to `read_dir` the site directory")? {
        let entry = entry.context("Unable to read some entry when reading site directory item")?;

        /// For example, `.git` should not be deleted
        fn is_path_to_keep(path: &Path) -> bool {
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => return false,
            };
            name.starts_with(".")
        }

        let path = entry.path();

        // never remove the temporary output directory
        if path == out_dir {
            continue;
        }

        if is_path_to_keep(&path) {
            continue;
        }

        if path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            //
        }
    }

    // copy the `includes` files in `book.ron` to the temporary output directory
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
        let entry = entry.context("Unable to read some entry when reading tmp directory item")?;

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
