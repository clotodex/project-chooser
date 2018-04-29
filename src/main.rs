/* dir to search in (default ~/projects)
 *
 * search query
 * if exact match => cd to there
 * if multiple matches => show menu to select
 * if no match => show message
 *
 * basename mode on/off
 * option to count exact basename match in non basename mode
 *
 * use a cache file
 *
 * make highly parallel with channels and a threadpool
 *
 * a match is:
 * - folder contains .git or .project
 *
 * optimize search:
 * - ignore .git folders
 * - ignore folders marked as .noproject
 * - optionally src/ directories etc
 * - [advanced] ignore all files/folders of a .projectignore
 *
 * OR do not decend in folders with .project => rather have .projectgroup
 *
 * future:
 * - let query be piped in
 * - config for adding ignored directories
 * - max depth for descending
 */

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

// one possible implementation of walking a directory only visiting files
fn visit_dirs<PMF, PIF, PICF>(dir: &Path, cb: &Fn(&Path), pred_match: &PMF, pred_ignored: &PIF, pred_ignore_current: &PICF) -> io::Result<()> 
where PMF: Fn(&DirEntry) -> bool + Send + Sync + 'static,
      PIF: Fn(&DirEntry) -> bool + Send + Sync + 'static,
      PICF: Fn(&DirEntry) -> bool + Send + Sync + 'static {
    if dir.is_dir() {
        let mut to_check : Vec<DirEntry> = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if pred_match(&entry) {
                cb(&dir);
            }
            if pred_ignored(&entry) {
                //do not visit dir
                continue;
            }
            if pred_ignore_current(&entry) {
                //clear todo list and ignore
                to_check.clear();
                break;
            }
            to_check.push(entry);
        }

        for entry in to_check {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb, pred_match, pred_ignored, pred_ignore_current)?;
            }
        }
    }
    Ok(())
}

fn callback(e: &Path) {
    println!("{:?}",e);
}

fn main() {
    let ok_path = vec![".git".to_string(), ".project".to_string()];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".noproject".to_string()];

    visit_dirs(Path::new("/home/clotodex/projects/"), &callback, &move |entry: &DirEntry| {
        return ok_path.contains(&entry.file_name().into_string().unwrap());
    }, &move |entry: &DirEntry| {
        return ignore_path_ends.contains(&entry.file_name().into_string().unwrap())
    }, &move |entry: &DirEntry| {
        return ignore_current.contains(&entry.file_name().into_string().unwrap())
    } ).unwrap()
}
