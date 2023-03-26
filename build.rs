use std::collections::VecDeque;
use std::path::{Path, PathBuf};

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/themes");
    if Path::new(".git").exists() {
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
    if let Ok(s) = std::fs::read_to_string(".cargo_vcs_info.json") {
        const KEY: &str = "\"sha1\":";

        fn find_tail<'str>(str: &'str str, tok: &str) -> Option<&'str str> {
            let i = str.find(tok)?;
            Some(&str[(i + tok.len())..])
        }

        if let Some(mut tail) = find_tail(&s, KEY) {
            while !tail.starts_with('"') && !tail.is_empty() {
                tail = &tail[1..];
            }
            if !tail.is_empty() {
                // skip "
                tail = &tail[1..];
                if let Some(end) = find_tail(tail, "\"") {
                    let end = tail.len() - end.len() - 1;
                    println!("cargo:rustc-env=PACKAGE_GIT_SHA={}", &tail[..end]);
                }
            }
        }
    }
    build_info_build::build_script();
    Ok(())
}
