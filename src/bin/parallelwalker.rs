use ignore::DirEntry as DE;
use ignore::WalkBuilder;
use ignore::WalkState;
use ignore::ParallelVisitor;
use ignore::ParallelVisitorBuilder;
use ignore::Error;
use project_chooser::walker;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};
use std::time::{Instant, Duration};
use std::sync::mpsc;

struct MyWalkFnBuilder {
    tx: mpsc::Sender<Result<DE,Error>>,
    stoppers: Vec<String>,

}

type FnVisitor<'s> =
    Box<dyn FnMut(Result<DE, Error>) -> WalkState + Send + 's>;

impl<'s> MyWalkFnBuilder {
    fn create_visitor(&mut self) -> FnVisitor<'s> {
        let txx = mpsc::Sender::clone(&self.tx);
        let fnstoppers = self.stoppers.clone();
        return Box::new(move |path: Result<DE, Error>| { if fnstoppers
            .iter()
            .any(|s| path.as_ref().unwrap().path().join(s).exists())
        {
            txx.send(path).unwrap();
            WalkState::Skip
        } else {
            if path.as_ref().unwrap().path().join(".groupproject").exists() {
                txx.send(path).unwrap();
            }
            WalkState::Continue
        }
    });

    }
}

impl<'s> ParallelVisitorBuilder<'s> for MyWalkFnBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(FnVisitorImp { visitor: self.create_visitor() })
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

fn main() {

    let showstoppers = vec![
        ".git".to_string(), /*, "src".to_string()*/
        ".project".to_string(),
    ];
    let stoppers = showstoppers.clone();

    let mut paths: Vec<PathBuf> = vec![];
    let (tx, rx) = mpsc::channel();

    let start = Instant::now();
    WalkBuilder::new("/home/clotodex/projects/")
        .hidden(false)
        .parents(false)
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .follow_links(false)
        .filter_entry(|entry: &DE| entry.file_type().unwrap().is_dir())
        .build_parallel()
        .visit(&mut MyWalkFnBuilder{tx, stoppers});

    for path in &rx {
        paths.push(path.as_ref().unwrap().path().to_path_buf());
    }
    let duration = start.elapsed();
    println!("Time elapsed in IgnoreWalkerParallel() is: {:?}", duration);

    let ok_path = vec![
        ".git".to_string(),
        ".project".to_string(),
        ".groupproject".to_string(),
    ];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".project".to_string(), ".git".to_string()]; //TODO this is what beats a normal find

    let mut paths: Vec<PathBuf> = vec![];

    let start = Instant::now();
    walker::visit_dirs(
        Path::new("/home/clotodex/projects/"),
        &mut |p: &Path| {
            paths.push(p.to_path_buf());
        },
        &move |entry: &DirEntry| {
            //return ok_path.contains(&entry.file_name().into_string().unwrap());
            ok_path.contains(&entry.file_name().into_string().unwrap_or_else(|_x| {
                /*println!("{:?}",x);*/
                "".to_string()
            }))
        },
        &move |entry: &DirEntry| {
            ignore_path_ends.contains(&entry.file_name().into_string().unwrap_or_else(|_x| {
                /* println!("{:?}",x); */
                "".to_string()
            }))
        },
        &move |entry: &DirEntry| {
            ignore_current.contains(&entry.file_name().into_string().unwrap_or_else(|_x| {
                /* println!("{:?}",x); */
                "".to_string()
            }))
        },
    )
    .unwrap();
    let duration = start.elapsed();
    println!("Time elapsed in Walker::visit_dirs() is: {:?}", duration);
}
