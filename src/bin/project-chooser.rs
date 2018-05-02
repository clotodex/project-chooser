// add tags / group by project type (rust projects have Cargo.toml
extern crate rust_project_chooser;

use rust_project_chooser::{walker, search::SearchKind};
use std::{fs::DirEntry, path::{Path, PathBuf}};
use std::process::{Command, Stdio};
use std::{io, io::prelude::*};
use std::error::Error;

enum OutputMode {
    ALL,
    FIRST,
    INTERACTIVE,
}

struct ProgramOptions {
    root: PathBuf,
    search_kind: SearchKind,
    mode: OutputMode,
    verbose: bool,
    threads: Option<u16>,
    use_cache: bool,
    query: Option<String>,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        ProgramOptions {
            root: {
                let mut home = std::env::home_dir().unwrap();
                home.push("projects");
                home
            },
            search_kind: SearchKind::FULL,
            mode: OutputMode::INTERACTIVE,
            verbose: false,
            threads: None,
            use_cache: false,
            query: None,
        }
    }
}

fn dmenu_search(paths: Vec<PathBuf>, search_kind: SearchKind) -> Vec<PathBuf> {
    // Spawn the `wc` command
    // dmenu -i -l 100 -p 'open project:'
    let process = match Command::new("dmenu")
        .args(&["-i", "-l", "100", "-p", "open project:"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn() {
        Err(why) => panic!("couldn't spawn dmenu: {}", why.description()),
        Ok(process) => process,
    };

    //TODO could also loop over iterator and build string directly
    //TODO maybe append string with its index with enumerate => super simple result extraction
    let strings: Vec<String> = paths.iter().map(|p| {
        match search_kind {
            SearchKind::BASENAME => p.file_name().unwrap().to_string_lossy().to_string(),
            SearchKind::FULL => p.to_string_lossy().to_string()
        }
    }).collect();
    let string = strings.join("\n");


    //need to drop instream so outstream is active
    match process.stdin.unwrap().write_all(string.as_bytes()) {
        Err(why) => panic!("couldn't write to dmenu stdin: {}",
                          why.description()),
        Ok(_) => println!("sent paths to dmenu"),
    }

    // The `stdout` field also has type `Option<ChildStdout>` so must be unwrapped.
    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(why) => panic!("couldn't read dmenu stdout: {}",
                           why.description()),
        Ok(_) => print!("dmenu responded with:\n{}", s),
    }

    s.pop();
    return match search_kind {
        SearchKind::FULL => vec![PathBuf::from(&s)],
        SearchKind::BASENAME => paths.into_iter().filter(|p| p.file_name().unwrap().to_string_lossy() == s).collect()
    }
}

fn rust_search(paths: Vec<PathBuf>, search_kind: SearchKind, query: &str) -> Vec<PathBuf> {
    return paths
        .into_iter()
        .filter(|p| search_kind.search(p, &query))
        .collect();
}

fn main() {
    //TODO make query pipeable in stdin => path collecting only once => search reuse

    // read input arguments
    // - direct serach
    // - basename / nobasename
    // - outputall or choose best or interactive
    // - verbose mode for discovering what indexing is slow
    // - numthreads or seqential
    // - where to search
    let mut options = ProgramOptions::default();

    //tmp
    options.search_kind = SearchKind::BASENAME;
    //options.query = Some("cLoUd".to_string());

    if options.verbose {
        unimplemented!("verbose mode not available yet");
    }
    if let Some(_) = options.threads {
        unimplemented!("multi threading is not available yet");
    }
    if options.use_cache {
        unimplemented!("cache is not available yet");
    }

    // retrieve project paths
    let ok_path = vec![".git".to_string(), ".project".to_string()];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".noproject".to_string()];

    //TODO use channel for live updates - multithread
    let mut paths: Vec<PathBuf> = vec![];

    walker::visit_dirs(
        &options.root,
        &mut |p: &Path| {
            paths.push(p.to_path_buf());
        },
        &move |entry: &DirEntry| {
            return ok_path.contains(&entry.file_name().into_string().unwrap());
        },
        &move |entry: &DirEntry| {
            return ignore_path_ends.contains(&entry.file_name().into_string().unwrap());
        },
        &move |entry: &DirEntry| {
            return ignore_current.contains(&entry.file_name().into_string().unwrap());
        },
    ).unwrap();

    println!("found {} total projects", paths.len());
    let results: Vec<PathBuf> = if let Some(query) = options.query {
        rust_search(paths, options.search_kind, &query)
    } else {
        dmenu_search(paths, options.search_kind)
    };

    if results.len() == 0 {
        panic!("no results found!");
    }

    println!("query matched {} projects", results.len());

    match options.mode {
        OutputMode::ALL => {
            for r in &results {
                println!("{:?}", r);
            }
        },
        OutputMode::FIRST => println!("{:?}", results[0]),
        OutputMode::INTERACTIVE => {
            if results.len() == 1 {
                println!("{:?}", results[0])
            } else {
                println!();
                println!("Please type a number between 0 and {} to choose a project", results.len()-1);
                for (i,r) in results.iter().enumerate() {
                    println!("[{}] {:?}",i, r);
                }

                let num: usize =
                loop {
                    let mut input = String::new();
                    print!("input number: ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();
                    if let Ok(x) = input.parse::<usize>() {
                        break x;
                    } else {
                        println!("please type in a number or press ctrl-c to quit");
                    }
                };

                println!("{:?}", results[num]);
            }
        },
    }
}
