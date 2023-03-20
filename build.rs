use std::collections::VecDeque;
use std::path::{Path, PathBuf};

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/themes");
    {
        fn read_fn(p: impl AsRef<Path>) -> Result<VecDeque<PathBuf>, std::io::Error> {
            std::fs::read_dir(p)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<VecDeque<_>, std::io::Error>>()
        }
        let theme_dirs = read_fn("src/themes/")?;
        for theme in theme_dirs.into_iter().filter(|p| p.is_dir()) {
            let theme_css = theme.join("gtk.css");
            let mdata = std::fs::metadata(&theme_css)?.modified()?;
            let mut entries = read_fn(&theme)?;
            while let Some(p) = entries.pop_front() {
                if p.is_dir() {
                    entries.extend(read_fn(&p)?.drain(..));
                    continue;
                }
                if p.extension() == Some("scss".as_ref()) && p.metadata()?.modified()? > mdata {
                    println!(
                        "cargo:warning=Theme {} might need to be recompiled from .scss sources:",
                        theme.file_name().unwrap().to_string_lossy()
                    );
                    println!("cargo:warning=File {} has a newer modified time than the compiled .css, {}.", p.display(), theme_css.display());
                }
            }
        }
    }
    #[cfg(feature = "build-info")]
    build_info_build::build_script();
    Ok(())
}
