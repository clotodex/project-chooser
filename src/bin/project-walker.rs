use project_chooser::walker;
use std::fs::DirEntry;
use std::path::Path;

fn callback(e: &Path) {
    println!("{:?}", e);
}

fn main() {
    let ok_path = vec![".git".to_string(), ".project".to_string(), ".groupproject".to_string()];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    //TODO should .git be in this? => .project is more "manual"
    //.noproject is deprecated
    let ignore_current = vec![".project".to_string(), ".git".to_string()];

    println!();
    println!("--- NEW ---");
    walker::visit_dirs(
        Path::new("/home/clotodex/projects/"),
        &mut callback,
        &move |entry: &DirEntry| {
            ok_path.contains(&entry.file_name().into_string().unwrap())
        },
        &move |entry: &DirEntry| {
            ignore_path_ends.contains(&entry.file_name().into_string().unwrap())
        },
        &move |entry: &DirEntry| {
            ignore_current.contains(&entry.file_name().into_string().unwrap())
        },
    ).unwrap();
}
