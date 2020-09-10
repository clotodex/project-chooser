// add tags / group by project type (rust projects have Cargo.toml

#[macro_use]
extern crate log;

use project_chooser::{
    cache::Cache,
    walker::{self, ProjectGathererInfo},
};
use std::path::PathBuf;

fn main() {
    debug!("cache");

    let mut root = dirs::home_dir().unwrap();
    root.push("projects");

    let mut cache_file = dirs::cache_dir().unwrap();
    cache_file.push("project-chooser.cache");
    let mut cache = Cache::load(&cache_file).unwrap();

    let mut paths: Vec<PathBuf> = vec![];
    for p in &walker::visit_dirs(root, ProjectGathererInfo::default()) {
        paths.push(p.as_ref().unwrap().path().to_path_buf());
    }
    cache.update(&paths);
    cache.save().unwrap();
}
