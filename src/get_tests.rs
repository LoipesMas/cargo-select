use std::{
    io::{BufRead, BufReader},
    path::Path,
};

use walkdir::{DirEntry, WalkDir};

use crate::select::{Target, TestTarget};

fn is_rust_source_or_dir(dir_entry: &DirEntry) -> bool {
    log::trace!("is_rust_source_or_dir {dir_entry:?}");
    dir_entry
        .file_name()
        .to_str()
        .map(|s| {
            s.ends_with(".rs")
                || (dir_entry.file_type().is_dir()
                    && dir_entry
                        .file_name()
                        .to_str()
                        .map(|s| s != "target")
                        .unwrap_or(true))
        })
        .unwrap_or(false)
}

fn get_tests_from_file(dir_entry: &DirEntry) -> Vec<Target> {
    log::debug!("Getting tests from file: {dir_entry:?}");
    assert!(dir_entry.file_type().is_file());
    let path = dir_entry.path().to_path_buf();
    let mut tests = vec![];
    let file = std::fs::File::open(dir_entry.path()).expect("Couldn't open file lol");
    let mut find_test_function = false;
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line == "#[test]" {
            find_test_function = true;
        } else if find_test_function {
            //TODO: this still can miss things like:
            // ```
            // #[test]
            // fn
            // foo
            // ()
            // {}
            // ```
            if let Some(line) = line
                .strip_prefix("fn")
                .or_else(|| line.strip_prefix("pub fn"))
            {
                // get just the name of the function
                let i = line
                    .find('(')
                    .unwrap_or_else(|| line.len().saturating_sub(1));
                let name = line[..i].trim().to_owned();

                log::trace!("Found test: {name}");

                tests.push(Target::Test(TestTarget {
                    name,
                    path: path.clone(),
                }));

                find_test_function = false;
            }
        }
    }
    tests
}

pub fn get_tests_from_path(path: &Path) -> Vec<Target> {
    log::debug!("Getting tests recursively from path: {path:?}");
    let mut tests = vec![];

    let walker = WalkDir::new(path).into_iter();
    //TODO: multithreading?
    for entry in walker.filter_entry(is_rust_source_or_dir) {
        if entry.as_ref().unwrap().file_type().is_file() {
            tests.append(&mut get_tests_from_file(entry.as_ref().unwrap()));
        }
    }
    tests
}
