/*!
Internal utilities
*/

use {
    anyhow::{Context, Result},
    colored::*,
    std::{fs, path::Path},
};

/// Prints printables as errors with a header
pub fn print_errors(errors: &Vec<impl std::fmt::Display>, header_text: &str) {
    self::print_items(errors, "error", header_text);
}

/// Prints printables as errors with a header
pub fn print_warnings(warnings: &Vec<impl std::fmt::Display>, header_text: &str) {
    self::print_items(warnings, "warnings", header_text);
}

fn print_items(items: &Vec<impl std::fmt::Display>, kind_name: &str, header_text: &str) {
    if items.is_empty() {
        return;
    }

    // header string: "<n> error[s] <header_text>"
    let kind_name = if items.len() == 1 {
        kind_name.to_string()
    } else {
        format!("{}s", kind_name)
    };

    let text = format!("{} {} {}:", items.len(), kind_name, header_text);
    eprintln!("{}", text.red().bold());

    for item in items {
        eprintln!("- {}", item);
    }
}

/// Copies items in one directory to another recursively
pub fn copy_items_rec(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    // log::trace!(
    //     "Recursive copy: `{}` -> `{}`",
    //     src_dir.display(),
    //     dst_dir.display(),
    // );

    ensure!(
        src_dir != dst_dir,
        "Same source/destination when trying recursive copy!: {}",
        src_dir.display()
    );

    ensure!(
        src_dir.exists() && src_dir.is_dir(),
        "Given invalid source path to copy their items recursively"
    );

    if !dst_dir.exists() {
        fs::create_dir(dst_dir)
            .with_context(|| "Can't create destination directory on recursive copy")?;
    } else {
        ensure!(
            dst_dir.is_dir(),
            "Some non-directory item exists to the destination path on recursive copy: {}",
            dst_dir.display(),
        );
    }

    self::copy_items_rec_impl(src_dir, dst_dir).with_context(|| "Error when trying recursive copy")
}

fn copy_items_rec_impl(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;

        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src_dir).unwrap();
        let dst_path = dst_dir.join(rel_path);

        if src_path.is_file() {
            // case 1. file: just copy
            // log::trace!(
            //     "- copy file: `{}` -> `{}`",
            //     src_path.display(),
            //     dst_path.display()
            // );

            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            // case 2. directory: recursive copy
            // log::trace!(
            //     "- copy dir: `{}` -> `{}`",
            //     src_path.display(),
            //     dst_path.display()
            // );

            if !dst_path.exists() {
                fs::create_dir(&dst_path)
                    .with_context(|| "Unable to create directory on recursive copy")?;
            }

            self::copy_items_rec_impl(&src_path, &dst_path)?;
        } else {
            // case 3. unexpected kind of item: error
            eprintln!(
                "Unexpected kind of item when doing recursive copy: {}",
                src_path.display()
            );
        }
    }

    Ok(())
}

pub fn clear_directory_items(dir: &Path, is_path_to_keep: impl Fn(&Path) -> bool) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if is_path_to_keep(&path) {
            continue;
        }

        if path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            debug!(
                "clear: skipping unexpected kind of item: {}",
                path.display()
            );
        }
    }

    Ok(())
}

/// Recursively runs given procedure to files under a directory
///
/// The user procedure takes absolute path as a parameter. Stops immediately when any error is
/// found.
pub fn visit_files_rec(dir: &Path, f: &mut impl FnMut(&Path) -> Result<()>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let entry_path = dir.join(entry.path());

        if entry_path.is_file() {
            f(&entry_path)?;
        } else if entry_path.is_dir() {
            self::visit_files_rec(&entry_path, f)?;
        } else {
            log::trace!("Skipping unexpected kind of file: {}", entry_path.display());
        }
    }

    Ok(())
}
