/*!
Generates `all.adoc`
*/

use std::{fmt::Write, path::Path};

use crate::book::{
    toc::{Toc, TocItem},
    BookStructure,
};

type Result<T> = std::result::Result<T, std::fmt::Error>;

/// Generates `all.adoc`
///
/// FIXME: footnote
pub fn gen_all(book: &BookStructure) -> Result<String> {
    let mut out = String::new();

    writeln!(out, ":stylesheet: all.css")?;
    writeln!(out, "")?;

    self::visit(&mut out, &book.toc, 1)?;

    Ok(out)
}

fn visit(out: &mut String, toc: &Toc, depth: usize) -> Result<()> {
    // NOTE: We won't convert index.adoc since it should be included in the sidebar
    // self::write_file(out, &toc.summary, depth)?;

    let depth = depth + 1;

    for item in &toc.items {
        match item {
            TocItem::File(_name, abs_path) => {
                self::write_file(out, abs_path, depth)?;
            }
            TocItem::Dir(toc) => {
                self::visit(out, toc, depth)?;
            }
        }
    }

    Ok(())
}

fn write_file(out: &mut String, file: &Path, depth: usize) -> Result<()> {
    writeln!(out, "include::{}[leveloffset={}]", file.display(), depth)
}

// include::snowrl/summary.adoc[leveloffset=1]
// include::snowrl/1_batcher.adoc[leveloffset=2]
// include::snowrl/2_blur.adoc[leveloffset=2]
//
// include::rl/summary.adoc[leveloffset=1]
// include::rl/1_wfc.adoc[leveloffset=2]
