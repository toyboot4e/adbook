//! Utilities

use {
    anyhow::{Context, Result},
    colored::*,
    std::{fs, path::Path},
};

pub fn print_errors(errors: &Vec<impl std::fmt::Display>, header_text: &str) {
    if errors.is_empty() {
        return;
    }

    // header string: "<n> error[s] <header_text>"
    eprintln!(
        "{} {}:",
        format!(
            "{} {}",
            errors.len(),
            if errors.len() == 0 { "error" } else { "errors" }
        )
        .red(),
        header_text.red()
    );

    for err in errors {
        eprintln!("- {}", err);
    }
}

pub fn copy_items_rec(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    log::trace!(
        "Recursive copy: `{}` -> `{}`",
        src_dir.display(),
        dst_dir.display(),
    );

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
            log::trace!(
                "* file: `{}` -> `{}`",
                src_path.display(),
                dst_path.display()
            );

            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            // case 2. directory: recursive copy
            log::trace!(
                "* dir: `{}` -> `{}`",
                src_path.display(),
                dst_path.display()
            );

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
