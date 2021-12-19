/*!
Book builder
*/

pub mod cache;
pub mod convert;
pub mod visit;

use std::{fs, path::Path};

use anyhow::*;

use crate::{
    book::{walk, BookStructure},
    build::{cache::CacheIndex, visit::AdocBookBuilder},
    utils,
};

/// Builds an `adbook` structure into a site directory, making use of cache and parallelization
///
/// `src` -> `tmp` -> `site`
pub fn build_book(book: &BookStructure, force_rebuild: bool, log: bool) -> Result<()> {
    let site_dir = book.site_dir_path();
    utils::validate_dir(&site_dir)
        .with_context(|| format!("Failed to create site directory at: {}", site_dir.display()))?;

    let index = if force_rebuild {
        CacheIndex::empty()
    } else {
        cache::CacheIndex::load(book)?
    };

    // TODO: Generate in parallel
    // // 1. generate `all.adoc`
    // if book.book_ron.generate_all {
    //     let all =
    //         crate::build::convert::gen_all(book).with_context(|| "Unable to create `all.adoc`")?;

    //     let path = book.book_ron.src_dir.join("all.adoc");

    //     // overwrite only when it changed
    //     match fs::read(&path) {
    //         Ok(s) if s == all.as_bytes() => {}
    //         _ => {
    //             fs::write(path, all)?;
    //         }
    //     }
    // }

    // 2. build the project
    let (mut builder, errors) = AdocBookBuilder::from_book(book, index.create_diff(book)?)?;
    utils::print_errors(&errors, "while creating AdocBookVisitor");

    // ensure `asciidoctor` is in user PATH
    if which::which("asciidoctor").is_err() {
        bail!("`asciidoctor` is not in PATH");
    }

    log::info!("---- Running builders");
    let outputs = walk::walk_book_await_collect(&mut builder, &book, log);

    // 3. copy the outputs to the site directory
    log::info!("---- Writing to site directory");
    {
        let mut errors = Vec::new();
        let res = self::create_site_directory(&outputs, book, &book.site_dir_path(), &mut errors);
        utils::print_errors(&errors, "while copying temporary files to site directory");
        res?;
    }

    // 4. apply `copies` attribute
    log::info!("---- Copying specified files");
    {
        let root = &book.root;

        let mut errors = Vec::new();
        let mut warns = Vec::new();

        for (a, b) in &book.book_ron.copies {
            let src = root.join(a);
            let dst = root.join(b);

            if !src.exists() {
                warns.push(format!("Non-existing source file: {}", src.display()));
                continue;
            }

            // create directory in destination
            {
                let dir = if src.is_file() {
                    dst.parent()
                } else if src.is_dir() {
                    Some(dst.as_path())
                } else {
                    warns.push(format!("Unexpected kind of item: {}", src.display()));
                    continue;
                };

                if let Some(dir) = dir {
                    if !dir.exists() {
                        if let Err(err) = fs::create_dir_all(&dir)
                            .map_err(|err| anyhow!("{} (fs::create_dir({}))", err, dir.display()))
                        {
                            errors.push(err);
                            continue;
                        }
                    }
                }
            }

            if src.is_file() {
                if let Err(err) = fs::copy(&src, &dst).map_err(|err| {
                    anyhow!("{} (fs::copy({}, {}))", err, src.display(), dst.display())
                }) {
                    errors.push(err);
                }
            } else if src.is_dir() {
                if let Err(err) = utils::copy_items_rec(&src, &dst) {
                    errors.push(err);
                }
            }
        }

        utils::print_warnings(&warns, "while applying `copies` attribute");
        utils::print_errors(&errors, "while applying `copies` attribute");
    }

    // 5. apply `use_default_theme` attributes
    if book.book_ron.use_default_theme {
        log::info!("---- Copying default theme");
        crate::book::init::copy_default_theme(&site_dir)?;
    }

    // 6. clean up and save cache
    log::info!("---- Updating build cache");

    // copy outputs to the cache directory
    {
        let cache_dir = CacheIndex::locate_cache_dir(book)?;
        let mut errors = Vec::new();
        self::write_html_outputs(&mut errors, &book.src_dir_path(), &cache_dir, &outputs)?;
        utils::print_errors(&errors, "while writing outputs to cache");
    }

    index.update_cache_index(book, builder.cache_diff.into_new_cache_data())?;

    Ok(())
}

/// TODO: refactor
fn create_site_directory(
    outputs: &[walk::BuildOutput],
    book: &BookStructure,
    out_dir: &Path,
    errors: &mut Vec<Error>,
) -> Result<()> {
    let site_dir = book.site_dir_path();

    // clear most files in site directory
    log::trace!("remove files in site directory");
    utils::clear_directory_items(&site_dir, |path| {
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

            utils::copy_items_rec(&src_path, &dst_path).with_context(|| {
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

    // finally, copy the output (HTML) files to the site directory
    let src_dir = book.src_dir_path();
    let site_dir = book.site_dir_path();
    self::write_html_outputs(errors, &src_dir, &site_dir, outputs)?;

    Ok(())
}

fn write_html_outputs(
    errors: &mut Vec<Error>,
    src_dir: &Path,
    out_dir: &Path,
    outputs: &[walk::BuildOutput],
) -> Result<()> {
    for output in outputs {
        let dst_path = {
            let src_file = output.src_file.with_extension("html");
            let rel_path = src_file.strip_prefix(&src_dir).unwrap();
            out_dir.join(rel_path)
        };

        println!("{}", dst_path.display());

        let dir = dst_path.parent().unwrap();

        if !dir.exists() {
            if let Err(err) = fs::create_dir_all(&dir) {
                errors.push(anyhow!(
                    "Unable to create directory: {} (IO error: {})",
                    dir.display(),
                    err
                ));
                continue;
            }
        }

        if !dir.is_dir() {
            errors.push(anyhow!("Non-directory: `{}`", dir.display()));
            continue;
        }

        fs::write(&dst_path, &output.string)?;
    }

    Ok(())
}
