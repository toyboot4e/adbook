//! Book builder

pub mod convert;
pub mod visit;

use {
    anyhow::{Context, Error, Result},
    futures::executor::block_on,
    std::{fs, path::Path},
};

use crate::{
    book::{walk::walk_book_async, BookStructure},
    build::visit::AdocBookVisitor,
};

/// Builds an `adbook` structure into a site directory
///
/// book -> tmp -> site
pub fn build_book(book: &BookStructure) -> Result<()> {
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
    let tmp_dir = book.site_dir_path().join("__adbook_tmp__");
    self::validate_tmp_dir(&tmp_dir).context("Unable to validate temporary output directory")?;

    // now let's build the project!
    let (mut v, errors) = AdocBookVisitor::from_book(book, &tmp_dir);
    crate::utils::print_errors(&errors, "while creating AdocBookVisitor");

    block_on(walk_book_async(&mut v, &book));

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

fn validate_tmp_dir(out_dir: &Path) -> Result<()> {
    // make sure we have an available temporary output directory
    if out_dir.exists() {
        ensure!(
            out_dir.is_dir(),
            "There's something that prevents `adbook` from making a temporary output directory at: {}",
            out_dir.display()
        );

        // OK, we'll remove the directory anyway
        trace!(
            "Removing the existing temporary output directory at: {}",
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

    Ok(())
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
    crate::utils::clear_directory_items(&site_dir, |path| {
        if path == out_dir {
            return true;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => return false,
        };
        name.starts_with(".")
    })?;

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
