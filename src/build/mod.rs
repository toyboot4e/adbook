//! Book builder

pub mod cache;
pub mod convert;
pub mod visit;

use {
    anyhow::{Context, Error, Result},
    futures::executor::block_on,
    std::{fs, path::Path},
};

use crate::{
    book::{walk::walk_book_async, BookStructure},
    build::{cache::CacheIndex, visit::AdocBookVisitor},
};

/// Builds an `adbook` structure into a site directory, making use of caches and parallelization
///
/// book -> tmp -> site
pub fn build_book(book: &BookStructure) -> Result<()> {
    let site_dir = book.site_dir_path();
    crate::utils::validate_dir(&site_dir)
        .with_context(|| format!("Failed to create site directory at: {}", site_dir.display()))?;

    let index = cache::CacheIndex::load(book)?;
    let new_cache_dir = CacheIndex::locate_new_cache_dir(book)?;

    // 1. build the project
    let (mut v, errors) =
        AdocBookVisitor::from_book(book, index.create_diff(book)?, &new_cache_dir);
    crate::utils::print_errors(&errors, "while creating AdocBookVisitor");
    info!("---- Running builders");
    block_on(walk_book_async(&mut v, &book));

    // 2. copy the build files to the site directory
    info!("---- Copying output files to site directory");
    {
        let mut errors = Vec::with_capacity(10);
        let res = self::overwrite_site_with_temporary_outputs(book, &new_cache_dir, &mut errors);
        crate::utils::print_errors(&errors, "while copying temporary files to site directory");
        res?;
    }

    // 3. clean up and save cache
    info!("---- Updating build cache");
    index.clean_up_and_save(book, v.cache_diff.into_new_cache_data())?;

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

    // copy the output files to the site directory
    for entry in fs::read_dir(&out_dir).context("Unable to `read_dir` the tmp directory")? {
        let entry = entry.context("Unable to read some entry when reading tmp directory item")?;

        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(out_dir).unwrap();
        let dst_path = site_dir.join(rel_path);

        if src_path.is_file() {
            trace!(
                "- copy `{}` -> `{}`",
                src_path.display(),
                dst_path.display()
            );
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
