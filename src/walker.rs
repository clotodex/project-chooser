use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

//TODO can be optimized to work on filename directly
pub fn visit_dirs<PMF, PIF, PICF>(dir: &Path, cb: &mut FnMut(&Path), pred_match: &PMF, pred_ignored: &PIF, pred_ignore_current: &PICF) -> io::Result<()>
where PMF: Fn(&DirEntry) -> bool + Send + Sync + 'static,
      PIF: Fn(&DirEntry) -> bool + Send + Sync + 'static,
      PICF: Fn(&DirEntry) -> bool + Send + Sync + 'static {
    let dir: &Path = &dir.canonicalize()?;
    if dir.is_dir() {
        let mut to_check : Vec<DirEntry> = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if pred_match(&entry) {
                //TODO can callback be a direntry by any chance?
                //TODO why does cb need to be &mut
                cb(&dir);
            }
            if pred_ignore_current(&entry) {
                //clear todo list and ignore
                to_check.clear();
                break;
            }
            if pred_ignored(&entry) {
                //do not visit dir
                continue;
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
