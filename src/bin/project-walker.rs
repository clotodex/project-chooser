extern crate rust_project_chooser;

use rust_project_chooser::walker;
use std::fs::{DirEntry};
use std::path::Path;

fn callback(e: &Path) {
    println!("{:?}",e);
}

fn main() {
    let ok_path = vec![".git".to_string(), ".project".to_string()];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".noproject".to_string()];

    walker::visit_dirs(Path::new("/home/clotodex/projects/"), &mut callback, &move |entry: &DirEntry| {
        return ok_path.contains(&entry.file_name().into_string().unwrap());
    }, &move |entry: &DirEntry| {
        return ignore_path_ends.contains(&entry.file_name().into_string().unwrap())
    }, &move |entry: &DirEntry| {
        return ignore_current.contains(&entry.file_name().into_string().unwrap())
    } ).unwrap()
}
