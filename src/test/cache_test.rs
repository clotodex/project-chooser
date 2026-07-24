use project_chooser::cache::Cache;
use std::path::{Path, PathBuf};

fn main() {
    let mut contents: Vec<PathBuf> = ["path/1", "path/2/", "path/3"]
        .iter()
        .map(PathBuf::from)
        .collect();
    println!("loading");
    let mut cache = Cache::load(&Path::new("test.cache")).unwrap();

    println!("cache: {:?}", cache.entries);
    cache.entries.clear();

    cache.update(&contents);
    println!("cache: {:?}", cache.entries);

    cache.select(&contents[1]);
    println!("cache: {:?}", cache.entries);
    cache.select(&contents[1]);
    println!("cache: {:?}", cache.entries);
    cache.select(&contents[1]);
    println!("cache: {:?}", cache.entries);
    cache.select(&contents[0]);
    println!("cache: {:?}", cache.entries);

    contents.pop();
    cache.update(&contents);
    println!("cache: {:?}", cache.entries);

    contents.push(PathBuf::from("path/4"));
    cache.update(&contents);
    println!("cache: {:?}", cache.entries);

    assert_eq!(
        cache.entries,
        vec![
            (1, PathBuf::from("path/1")),
            (3, PathBuf::from("path/2/")),
            (0, PathBuf::from("path/4"))
        ]
    );

    println!("selecting");
    cache.select(&PathBuf::from("/path/to/select"));
    println!("saving");
    cache.save().unwrap();
    println!("done");
}
