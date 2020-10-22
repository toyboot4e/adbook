//! Implementation of [`crate::book::walk::BookVisitor`]

use {
    anyhow::{Context, Result},
    std::{fs, path::Path},
};

use crate::book::{
    config::CmdOptions,
    walk::{BookVisitContext, BookVisitor},
};

/// An `adbook` builder based on `asciidoctor`
pub struct AdocBookVisitor {
    buf: String,
    opts: CmdOptions,
}

impl AdocBookVisitor {
    pub fn new(opts: CmdOptions) -> Self {
        AdocBookVisitor {
            opts,
            buf: String::with_capacity(1024 * 5),
        }
    }
}

impl BookVisitor for AdocBookVisitor {
    /// Gets destination path and kicks `asciidoctor` runner
    fn visit_file(&mut self, src_file: &Path, vcx: &mut BookVisitContext) -> Result<()> {
        match src_file.extension().and_then(|o| o.to_str()) {
            Some("adoc") => {}
            Some("md") => {
                bail!(".md file is not yet handled: {}", src_file.display());
            }
            _ => {
                bail!("Unexpected kind of file: {}", src_file.display());
            }
        }

        // relative path from source directory
        let rel = match src_file.strip_prefix(&vcx.src_dir) {
            Ok(r) => r,
            Err(_err) => bail!(
                "Fail that is not in source directly found: {}",
                src_file.display(),
            ),
        };

        let dst_file = vcx.dst_dir.join(&rel).with_extension("html");

        let dst_dir = dst_file.parent().with_context(|| {
            format!(
                "Failed to get parent directory of `.adoc` file: {}",
                src_file.display()
            )
        })?;

        if !dst_dir.is_dir() {
            fs::create_dir_all(&dst_dir).with_context(|| {
                format!(
                    "Failed to create parent directory of `.adoc` file: {}",
                    src_file.display(),
                )
            })?;
        }

        let dummy_dst_name = format!("{}", dst_file.display());
        self.buf.clear();
        crate::build::convert::convert_adoc_buf(
            &mut self.buf,
            src_file,
            &vcx.src_dir,
            &vcx.dst_dir,
            &dummy_dst_name,
            &self.opts,
        )?;
        // let mut acx = AdocContext::new(&vcx.src_dir, &vcx.dst_dir, &self.opts)?;
        // self.buf.clear();
        // adoc::run_asciidoctor_buf(src_file, &dummy_dst_name, &mut self.buf, &mut acx)?

        fs::write(&dst_file, &self.buf).with_context(|| {
            format!(
                "Unexpected error when trying to get access to destination file:\n  {}",
                dst_file.display(),
            )
        })?;

        Ok(())
    }
}
