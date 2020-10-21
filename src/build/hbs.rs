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

        let src_file = root.join("src/demo.adoc");
        let dst_file = man_dir.join("demo.html");
        let hbs = root.join("src/hbs/simple.hbs");

        let dst_dir = dst_file.parent().unwrap();
        let site_dir = dst_dir.to_path_buf();
        let dst_name = format!("{}", dst_file.display());

        // generate html text
        println!("test: {} => {}", src_file.display(), dst_file.display());
        let opts = vec![("--embedded".to_string(), vec![])];
        let text =
            crate::build::convert_adoc_with_hbs(&src_file, &site_dir, &dst_name, &hbs, opts)?;

        // output to stdout
        let out = io::stdout();
        let mut out = out.lock();
        out.write(text.as_bytes())?;

        Ok(())
    }
}
