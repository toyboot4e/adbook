/*!
Internal utilities
*/

use {
    anyhow::{Result, Context, ensure},
    colored::*,
    serde::de::DeserializeOwned,
    std::{fmt, fs, path::Path},
};

/// Load the given string as a RON format (or one without outermost parentheses)
pub fn load_ron<T>(s: &str) -> ron::de::Result<T>
where
    T: DeserializeOwned,
{
    match ron::de::from_str(&s) {
        Ok(data) => Ok(data),
        Err(why) => {
            // surround the text with parentheses and retry
            let s = format!("({})", s);

            match ron::de::from_str(&s) {
                Ok(data) => Ok(data),
                Err(_why) => Err(why),
            }
        }
    }
}

/// "N errors (header text):"
pub fn print_errors(errs: &[impl fmt::Display], header: &str) {
    self::print_items("error", errs, header);
}

/// "N warnings (header text):"
pub fn print_warnings(warns: &[impl fmt::Display], header: &str) {
    self::print_items("warning", warns, header);
}

fn print_items(kind: &str, items: &[impl fmt::Display], header: &str) {
    if items.is_empty() {
        return;
    }

    let kind = if items.len() == 1 {
        kind.to_string()
    } else {
        format!("{}s", kind)
    };

    let h = format!("{} {} {}:", items.len(), kind, header);
    eprintln!("{}", h.red().bold());

    for item in items {
        eprintln!("- {}", item);
    }
}

/// Copies all items in one directory to another recursively
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

/// Clears items just under the directory
pub fn clear_directory_items(dir: &Path, should_keep: impl Fn(&Path) -> bool) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if should_keep(&path) {
            continue;
        }

        if path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            log::debug!(
                "clear: skipping unexpected kind of item: {}",
                path.display()
            );
        }
    }

    Ok(())
}

/// Recursively runs given procedure to files just under the directory
///
/// The user procedure takes an absolute path as a parameter.
///
/// Stops immediately when any error is found.
pub fn visit_files_rec(dir: &Path, proc: &mut impl FnMut(&Path) -> Result<()>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let entry_path = dir.join(entry.path());

        if entry_path.is_file() {
            proc(&entry_path)?;
        } else if entry_path.is_dir() {
            self::visit_files_rec(&entry_path, proc)?;
        } else {
            log::trace!("Skipping unexpected kind of file: {}", entry_path.display());
        }
    }

    Ok(())
}

/// Creates or makes sure there's a directory
pub fn validate_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        fs::create_dir(dir)
            .with_context(|| format!("Unable to create directory at: {}", dir.display()))?;
    } else {
        ensure!(dir.is_dir(), "Non-directory item at {}", dir.display());
    }

    Ok(())
}
