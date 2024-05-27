use anyhow::Result;
use bundler::run_bundle;
use glob::{glob, GlobError};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::BUILD_DIR;

// get all files with certain extension in a directory
pub(crate) fn get_files_with_exts(dir: &str, exts: &[&str]) -> Result<BTreeSet<PathBuf>> {
    // glob all ts files
    let mut files = BTreeSet::new();
    for ext in exts {
        let rule = format!("{}/**/*.{}", dir, ext);
        let paths = glob(&rule)?.collect::<Result<BTreeSet<PathBuf>, GlobError>>()?;
        files.extend(paths);
    }
    Ok(files)
}

pub(crate) fn calc_project_hash(dir: &str) -> Result<String> {
    calc_hash_for_files(dir, &["ts", "js", "json"], 16)
}

pub(crate) fn calc_hash_for_files(dir: &str, exts: &[&str], len: usize) -> Result<String> {
    let files = get_files_with_exts(dir, exts)?;
    let mut hasher = blake3::Hasher::new();
    for file in files {
        hasher.update_reader(File::open(file)?)?;
    }
    let mut ret = hasher.finalize().to_string();
    ret.truncate(len);
    Ok(ret)
}

pub(crate) fn build_project(dir: &str) -> Result<String> {
    let hash = calc_project_hash(dir)?;
    fs::create_dir_all(BUILD_DIR)?;
    let filename = format!("{}/{}.mjs", BUILD_DIR, hash);
    let dst = Path::new(&filename);
    // if the file already exists, skip building
    if dst.exists() {
        return Ok(filename);
    }

    // build the project
    let content = run_bundle("main.ts", &Default::default())?;
    fs::write(dst, content)?;

    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_files_with_exts_should_work() -> Result<()> {
        let files = get_files_with_exts("fixtures/prj", &["ts", "js", "json"])?;
        assert_eq!(
            files.into_iter().collect::<Vec<_>>(),
            [
                PathBuf::from("fixtures/prj/a.ts"),
                PathBuf::from("fixtures/prj/test1/b.ts"),
                PathBuf::from("fixtures/prj/test1/c.js"),
                PathBuf::from("fixtures/prj/test2/test3/d.json"),
            ]
        );
        Ok(())
    }

    #[test]
    fn calc_hash_for_files_should_work() -> Result<()> {
        let hash = calc_hash_for_files("fixtures/prj", &["ts", "js", "json"], 12)?;
        assert_eq!(hash, "af1349b9f5f9");
        Ok(())
    }
}
