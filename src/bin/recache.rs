// add tags / group by project type (rust projects have Cargo.toml

#[macro_use]
extern crate log;

use project_chooser::{walker, cache::Cache};
use std::{fs::DirEntry, path::{Path, PathBuf}};

fn gather_projects(root: &Path) -> Vec<PathBuf> {
    // retrieve project paths
    let ok_path = vec![
        ".git".to_string(),
        ".project".to_string(),
        ".groupproject".to_string(),
    ];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".project".to_string(), ".git".to_string()]; //TODO this is what beats a normal find

    //TODO use channel for live updates - multithread
    let mut paths: Vec<PathBuf> = vec![];

    walker::visit_dirs(
        root,
        &mut |p: &Path| {
            paths.push(p.to_path_buf());
        },
        &move |entry: &DirEntry| {
            //return ok_path.contains(&entry.file_name().into_string().unwrap());
            ok_path.contains(&entry.file_name().into_string().unwrap_or_else(|_x| { /*println!("{:?}",x);*/ "".to_string() } ))
        },
        &move |entry: &DirEntry| {
            ignore_path_ends.contains(&entry.file_name().into_string().unwrap_or_else(|_x| { /* println!("{:?}",x); */ "".to_string() } ))
        },
        &move |entry: &DirEntry| {
            ignore_current.contains(&entry.file_name().into_string().unwrap_or_else(|_x| { /* println!("{:?}",x); */ "".to_string() } ))
        },
    ).unwrap();

    paths
}

// read input arguments
// - direct serach
// - basename / nobasename
// - outputall or choose best or interactive
// - verbose mode for discovering what indexing is slow
// - numthreads or seqential
// - where to search
fn main() {
    debug!("cache");

    let mut root = dirs::home_dir().unwrap();
    root.push("projects");

    let mut cache_file = dirs::cache_dir().unwrap();
    cache_file.push("project-chooser.cache");
    let mut cache = Cache::load(&cache_file).unwrap();

    let paths = gather_projects(&root);
    cache.update(&paths);
    cache.save().unwrap();
}
