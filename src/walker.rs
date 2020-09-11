use ignore::DirEntry as DE;
use ignore::Error;
use ignore::ParallelVisitor;
use ignore::ParallelVisitorBuilder;
use ignore::WalkBuilder;
use ignore::WalkState;
use std::path::Path;
use std::sync::mpsc;

struct MyWalkFnBuilder {
    tx: mpsc::Sender<Result<DE, Error>>,
    info: ProjectGathererInfo,
}

type FnVisitor<'s> = Box<dyn FnMut(Result<DE, Error>) -> WalkState + Send + 's>;

impl<'s> MyWalkFnBuilder {
    fn create_visitor(&mut self) -> FnVisitor<'s> {
        let txx = mpsc::Sender::clone(&self.tx);
        let info_clone = self.info.clone();
        return Box::new(move |path: Result<DE, Error>| {
            if info_clone
                .no_enter_echo
                .iter()
                .any(|s| path.as_ref().unwrap().path().join(s).exists())
            {
                txx.send(path).unwrap();
                WalkState::Skip
            } else if info_clone
                .no_enter
                .iter()
                .any(|s| path.as_ref().unwrap().path().join(s).exists())
            {
                WalkState::Skip
            } else {
                if info_clone
                    .enter_echo
                    .iter()
                    .any(|s| path.as_ref().unwrap().path().join(s).exists())
                {
                    txx.send(path).unwrap();
                }
                WalkState::Continue
            }
        });
    }
}

impl<'s> ParallelVisitorBuilder<'s> for MyWalkFnBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(FnVisitorImp {
            visitor: self.create_visitor(),
        })
    }
}

struct FnVisitorImp<'s> {
    visitor: FnVisitor<'s>,
}

impl<'s> ParallelVisitor for FnVisitorImp<'s> {
    fn visit(&mut self, entry: Result<DE, Error>) -> WalkState {
        (self.visitor)(entry)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectGathererInfo {
    no_enter: Vec<String>,
    no_enter_echo: Vec<String>,
    enter_echo: Vec<String>,
}
impl Default for ProjectGathererInfo {
    fn default() -> Self {
        ProjectGathererInfo {
            no_enter: vec!["src".to_owned()],
            no_enter_echo: vec![".git".to_owned(), ".project".to_owned()],
            enter_echo: vec![".groupproject".to_owned()],
        }
    }
}

pub fn visit_dirs<P: AsRef<Path>>(
    root: P,
    info: ProjectGathererInfo,
) -> mpsc::Receiver<Result<DE, Error>> {
    let (tx, rx) = mpsc::channel();
    WalkBuilder::new(root)
        .hidden(false)
        .parents(false)
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .follow_links(false)
        .filter_entry(|entry: &DE| entry.file_type().unwrap().is_dir())
        .build_parallel()
        .visit(&mut MyWalkFnBuilder { tx, info });
    rx
}
