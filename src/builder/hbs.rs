//! Handlebars application

#[cfg(test)]
mod test {
    use {
        anyhow::Result,
        std::io::{self, prelude::*},
    };

    #[test]
    fn test_article() -> Result<()> {
        let man_str = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let man_dir = std::path::PathBuf::from(man_str);

        let root = man_dir.join("samples/demo");

        let src = root.join("src/demo.adoc");
        let dst = man_dir.join("demo.html");
        let hbs = root.join("src/hbs/simple.hbs");

        // generate html text
        println!("test: {} => {}", src.display(), dst.display());
        let opts = vec![("--embedded".to_string(), vec![])];
        let text = crate::builder::convert_adoc_with_hbs(&src, &dst, &hbs, opts)?;

        // output to stdout
        let out = io::stdout();
        let mut out = out.lock();
        out.write(text.as_bytes())?;

        Ok(())
    }
}
