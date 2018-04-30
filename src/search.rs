use std::path::PathBuf;

pub enum SearchKind {
    BASENAME,
    FULL
}

//TODO implement for path or as iterator
impl SearchKind {
    //TODO use AsRef<Path>
    pub fn search(&self, p: &PathBuf, query: &str) -> bool {
        match *self {
            SearchKind::BASENAME => p.file_name().unwrap().to_string_lossy().contains(&query),
            SearchKind::FULL => p.to_string_lossy().contains(&query)
        }
    }    
}
