extern crate project_chooser;

use project_chooser::{
    walker,
    search::SearchKind
};
use std::{
    fs::{DirEntry},
    path::{Path, PathBuf}
};


fn main() {

    // gathering paths

    let ok_path = vec![".git".to_string(), ".project".to_string()];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".noproject".to_string()];

    //TODO use channel for live updates
    let mut paths: Vec<PathBuf> = vec![];

    walker::visit_dirs(Path::new("/home/clotodex/projects/"),
        &mut |p: &Path| {
            paths.push(p.to_path_buf());
        }, &move |entry: &DirEntry| {
            return ok_path.contains(&entry.file_name().into_string().unwrap());
        }, &move |entry: &DirEntry| {
            return ignore_path_ends.contains(&entry.file_name().into_string().unwrap())
        }, &move |entry: &DirEntry| {
            return ignore_current.contains(&entry.file_name().into_string().unwrap())
        }).unwrap();

    let query = "cloud".to_string();
    let mut search_kind = SearchKind::BASENAME;
    // search in results

    println!("search for {} in basename:", query);
    for b in paths.iter().filter(|p| search_kind.search(p, &query)) {
        println!("{:?}", b);
    }
    println!();

    println!("search for {} in full path:", query);
    search_kind = SearchKind::FULL;
    for b in paths.iter().filter(|p| search_kind.search(p, &query)) {
        println!("{:?}", b);
    }
}
