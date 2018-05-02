use std::path::PathBuf;

pub enum SearchKind {
    BASENAME,
    FULL
}

//TODO implement for path or as iterator
impl SearchKind {
    //TODO use AsRef<Path>
    pub fn search(&self, p: &PathBuf, query: &str) -> bool {
        //TODO make lowercase search adjustable - maybe even support regex
        let query = query.to_lowercase();
        match *self {
            SearchKind::BASENAME => p.file_name().unwrap().to_string_lossy().to_lowercase().contains(&query),
            SearchKind::FULL => p.to_string_lossy().to_lowercase().contains(&query)
        }
    }
}
