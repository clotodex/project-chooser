// add tags / group by project type (rust projects have Cargo.toml

#[macro_use]
extern crate log;
use stderrlog;
#[macro_use]
extern crate clap;

use project_chooser::{walker, search::SearchKind, cache::Cache};
use std::{fs::DirEntry, path::{Path, PathBuf}};
use std::process::{Command, Stdio};
use std::{io, io::prelude::*};
use std::error::Error;
use clap::Arg;
use std::str::FromStr;

arg_enum!{
    #[derive(PartialEq, Debug)]
    enum OutputMode {
        ALL,
        FIRST,
        INTERACTIVE
    }
}

#[derive(Debug)]
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
                let mut home = dirs::home_dir().unwrap();
                home.push("projects");
                home
            },
            search_kind: SearchKind::FULL,
            mode: OutputMode::INTERACTIVE,
            verbose: false,
            threads: None,
            use_cache: true,
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
        .spawn()
    {
        Err(why) => panic!("couldn't spawn dmenu: {}", why.description()),
        Ok(process) => process,
    };

    //TODO could also loop over iterator and build string directly
    //TODO maybe append string with its index with enumerate => super simple result extraction
    let strings: Vec<String> = paths
        .iter()
        .map(|p| match search_kind {
            SearchKind::BASENAME => p.file_name().unwrap().to_string_lossy().to_string(),
            SearchKind::FULL => p.to_string_lossy().to_string(),
        })
        .collect();
    let string = strings.join("\n");

    //need to drop instream so outstream is active
    match process.stdin.unwrap().write_all(string.as_bytes()) {
        Err(why) => panic!("couldn't write to dmenu stdin: {}", why.description()),
        Ok(_) => debug!("sent paths to dmenu"),
    }

    // The `stdout` field also has type `Option<ChildStdout>` so must be unwrapped.
    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(why) => panic!("couldn't read dmenu stdout: {}", why.description()),
        Ok(_) => debug!("dmenu responded with:\n{}", s),
    }

    s.pop();
    match search_kind {
        SearchKind::FULL => vec![PathBuf::from(&s)],
        SearchKind::BASENAME => paths
            .into_iter()
            .filter(|p| p.file_name().unwrap().to_string_lossy() == s)
            .collect(),
    }
}

fn rust_search(paths: Vec<PathBuf>, search_kind: SearchKind, query: &str) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|p| search_kind.search(p, &query))
        .collect()
}

fn parse_commandline_args() -> ProgramOptions {
    let mut options = ProgramOptions::default();
    let m = app_from_crate!()
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .help("Silence all output"),
        )
        .arg(
            Arg::with_name("path")
                .short("p")
                .takes_value(true)
                .help(&format!(
                    "Set root path for locating projects (default: {:?})",
                    options.root
                )),
        )
        .arg(
            //TODO remove SearchKind and just use a boolean flag
            Arg::with_name("basename")
                .short("b")
                .help("Search in path basenames instead of the whole path"),
        )
        .arg(
            Arg::with_name("cache")
                .short("c")
                .help("Cache all project paths for future use"),
        )
        .arg(
            //TODO switch to mutliple flags (-i and --first-only)
            Arg::with_name("mode")
                .short("m")
                .help(&format!(
                    "How to handle multiple results (default: {:?})",
                    options.mode
                ))
                .takes_value(true)
                .possible_values(&OutputMode::variants())
                .case_insensitive(true),
        )
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .help("Amount of threads to use (default: None)")
                .takes_value(true),
        )
        .arg(Arg::with_name("query").index(1))
        .get_matches();

    let verbose = m.occurrences_of("verbosity") as usize;
    let quiet = m.is_present("quiet");

    stderrlog::new()
        .module(module_path!())
        .module("project_chooser")
        .quiet(quiet)
        .verbosity(verbose)
        .init()
        .unwrap();

    if let Some(p) = m.value_of("path").map(PathBuf::from) {
        options.root = p
    };
    options.search_kind = if m.is_present("basename") {
        SearchKind::BASENAME
    } else {
        SearchKind::FULL
    };
    //options.mode = value_t!(m, "mode", OutputMode).unwrap_or_else(|e| e.exit());
    if let Some(mode) = m.value_of("mode").map(|m| {
        OutputMode::from_str(m).unwrap_or_else(|_| {
            clap::Error {
                message: "invalid value for 'mode'".into(),
                kind: clap::ErrorKind::InvalidValue,
                info: None,
            }.exit()
        })
    }) {
        options.mode = mode
    };
    options.threads = value_t!(m, "jobs", u16).ok();
    options.use_cache = m.is_present("cache") || options.use_cache;
    options.query = m.value_of("query").map(ToString::to_string);

    if options.threads.is_some() {
        unimplemented!("multi threading is not available yet");
    }
    options
}

fn gather_projects(root: &Path) -> Vec<PathBuf> {
    // retrieve project paths
    let ok_path = vec![
        ".git".to_string(),
        ".project".to_string(),
        ".groupproject".to_string(),
    ];
    let ignore_path_ends = vec![".git".to_string(), "src".to_string()];
    let ignore_current = vec![".project".to_string(), ".git".to_string()]; //TODO this is what beats a normal find

    //TODO use channel for live updates - multithread
    let mut paths: Vec<PathBuf> = vec![];

    walker::visit_dirs(
        root,
        &mut |p: &Path| {
            paths.push(p.to_path_buf());
        },
        &move |entry: &DirEntry| {
            //return ok_path.contains(&entry.file_name().into_string().unwrap());
            ok_path.contains(&entry.file_name().into_string().unwrap_or_else(|_x| { /*println!("{:?}",x);*/ "".to_string() } ))
        },
        &move |entry: &DirEntry| {
            ignore_path_ends.contains(&entry.file_name().into_string().unwrap_or_else(|_x| { /* println!("{:?}",x); */ "".to_string() } ))
        },
        &move |entry: &DirEntry| {
            ignore_current.contains(&entry.file_name().into_string().unwrap_or_else(|_x| { /* println!("{:?}",x); */ "".to_string() } ))
        },
    ).unwrap();

    paths
}

fn display_results(paths: Vec<PathBuf>, options: ProgramOptions){
    info!("found {} total projects", paths.len());
    let results: Vec<PathBuf> = if let Some(ref query) = options.query {
        rust_search(paths, options.search_kind, &query)
    } else {
        dmenu_search(paths, options.search_kind)
    };

    if results.is_empty() {
        panic!("no results found!");
    }

    info!("query matched {} projects", results.len());

    match options.mode {
        OutputMode::ALL => for r in &results {
            println!("{}", r.display());
        },
        OutputMode::FIRST => println!("{}", results[0].display()),
        OutputMode::INTERACTIVE => {
            if results.len() == 1 {
                println!("{}", results[0].display())
            } else {
                eprintln!("Multiple results!");
                eprintln!(
                    "Please type a number between 0 and {} to choose a project",
                    results.len() - 1
                );
                for (i, r) in results.iter().enumerate() {
                    eprintln!("[{}] {}", i, r.display());
                }

                let num: usize = loop {
                    let mut input = String::new();
                    eprint!("input number (0-{}): ", results.len()-1);
                    io::stderr().flush().unwrap();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();
                    match input.parse::<usize>() {
                        Ok(x) if x < results.len() => break x,
                        _ => eprintln!("please type a number inside the range 0 to {}", results.len()-1),
                    }
                };

                println!("{}", results[num].display());
            }
        }
    }
}

// read input arguments
// - direct serach
// - basename / nobasename
// - outputall or choose best or interactive
// - verbose mode for discovering what indexing is slow
// - numthreads or seqential
// - where to search
fn main() {
    //TODO make query pipeable in stdin => path collecting only once => search reuse
    //TODO replace all expects and unwraps with error!
    //TODO use builder macro from usage
    //TODO use exit codes
    //TODO use cache

    let options = parse_commandline_args();

    trace!("Arguments: {:?}", options);

    if options.use_cache {
        let mut cache_file = dirs::cache_dir().unwrap();
        cache_file.push("project-chooser.cache");
        let cache = Cache::load(&cache_file).unwrap();

        if cache.entries.is_empty() {
            warn!("cache file seems to be empty");
        }
        let results = cache.entries.iter().map(|&(_, ref e)| e.clone()).collect();
        display_results(results, options);
    } else {
        debug!("no cache");
        let paths = gather_projects(&options.root);
        display_results(paths, options);
    };
}
