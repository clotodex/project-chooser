// add tags / group by project type (rust projects have Cargo.toml

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::Arg;
use project_chooser::{cache::Cache, search::SearchKind, walker::{self, ProjectGathererInfo}};
use skim::prelude::*;
use std::error::Error;
use std::process;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::{
    path::{Path, PathBuf},
};
use std::{io, io::prelude::*};

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum OutputMode {
        ALL,
        FIRST,
        INTERACTIVE
    }
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum DisplayOption {
        SKIM,
        FZF,
        DMENU,
        ECHO,
    }
}

impl DisplayOption {
    fn display_search(&self, paths: Vec<PathBuf>, search_kind: SearchKind) -> Option<Vec<PathBuf>> {
        match self {
            DisplayOption::DMENU | DisplayOption::FZF => {
                let process = match self {
                    DisplayOption::DMENU => match Command::new("dmenu")
                        .args(&["-i", "-l", "100", "-p", "open project:"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        Err(why) => panic!("couldn't spawn dmenu: {}", why),
                        Ok(process) => process,
                    },
                    DisplayOption::FZF => match Command::new("fzf")
                        //.args(&["-i", "-l", "100", "-p", "open project:"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        Err(why) => panic!("couldn't spawn fzf: {}", why),
                        Ok(process) => process,
                    },
                    _ => unreachable!(),
                };

                //TODO could also loop over iterator and build string directly
                //TODO maybe append string with its index with enumerate => super simple result extraction
                let strings: Vec<String> = paths
                    .iter()
                    .map(|p| match search_kind {
                        SearchKind::BASENAME => {
                            p.file_name().unwrap().to_string_lossy().to_string()
                        }
                        SearchKind::FULL => p.to_string_lossy().to_string(),
                    })
                    .collect();
                let string = strings.join("\n");

                //need to drop instream so outstream is active
                match process.stdin.unwrap().write_all(string.as_bytes()) {
                    Err(why) => panic!("couldn't write to dmenu stdin: {}", why),
                    Ok(_) => debug!("sent paths to dmenu"),
                }

                // The `stdout` field also has type `Option<ChildStdout>` so must be unwrapped.
                let mut s = String::new();
                match process.stdout.unwrap().read_to_string(&mut s) {
                    Err(why) => panic!("couldn't read dmenu stdout: {}", why),
                    Ok(_) => debug!("dmenu responded with:\n{}", s),
                }

                s.pop();
                Some(match search_kind {
                    SearchKind::FULL => vec![PathBuf::from(&s)],
                    SearchKind::BASENAME => paths
                        .into_iter()
                        .filter(|p| p.file_name().unwrap().to_string_lossy() == s)
                        .collect(),
                })
            }
            DisplayOption::SKIM => {
                let options = SkimOptionsBuilder::default()
                    .height(Some("100%"))
                    .multi(true)
                    .build()
                    .unwrap();
                let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

                for p in paths.iter().map(|p| match search_kind {
                    SearchKind::BASENAME => p.file_name().unwrap().to_string_lossy().to_string(),
                    SearchKind::FULL => p.display().to_string(),
                }) {
                    let _ = tx_item.send(Arc::new(p.to_string()));
                }
                drop(tx_item); // so that skim could know when to stop waiting for more items.

                Some(
                    Skim::run_with(&options, Some(rx_item))
                        .map(|out| out.selected_items)
                        .map(|items| {
                            items
                                .into_iter()
                                .map(|i| PathBuf::from_str(&i.output()).unwrap())
                                .collect()
                        })
                        .unwrap_or_else(|| Vec::new()),
                )
            }
            DisplayOption::ECHO => {
                for p in paths.iter().map(|p| match search_kind {
                    SearchKind::BASENAME => p.file_name().unwrap().to_string_lossy().to_string(),
                    SearchKind::FULL => p.display().to_string(),
                }) {
                    println!("{}", p);
                }
                None
            }
        }
    }
}

#[derive(Debug)]
struct ProgramOptions {
    root: PathBuf,
    search_kind: SearchKind,
    mode: OutputMode,
    display: DisplayOption,
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
            display: DisplayOption::SKIM,
            verbose: false,
            threads: None,
            use_cache: true,
            query: None,
        }
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
            Arg::with_name("display")
                .short("d")
                .help(&format!("Display Tool (default: {:?})", options.display))
                .takes_value(true)
                .possible_values(&DisplayOption::variants())
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
    if m.is_present("mode") {
        options.mode = value_t!(m, "mode", OutputMode).unwrap_or_else(|e| e.exit());
    }
    if m.is_present("display") {
        options.display = value_t!(m, "display", DisplayOption).unwrap_or_else(|e| e.exit());
    }
    if let Some(mode) = m.value_of("mode").map(|m| {
        OutputMode::from_str(m).unwrap_or_else(|_| {
            clap::Error {
                message: "invalid value for 'mode'".into(),
                kind: clap::ErrorKind::InvalidValue,
                info: None,
            }
            .exit()
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
    let mut paths: Vec<PathBuf> = vec![];
    for p in &walker::visit_dirs(root, ProjectGathererInfo::default()) {
        paths.push(p.as_ref().unwrap().path().to_path_buf());
    }
    paths
}

fn display_results(paths: Vec<PathBuf>, options: ProgramOptions) -> Result<(), Box<dyn Error>> {
    info!("found {} total projects", paths.len());
    let results: Option<Vec<PathBuf>> = if let Some(ref query) = options.query {
        Some(rust_search(paths, options.search_kind, &query))
    } else {
        options.display.display_search(paths, options.search_kind)
    };

    if let Some(results) = results {
        if results.is_empty() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "no results found",
            )));
        }

        info!("query matched {} projects", results.len());

        match options.mode {
            OutputMode::ALL => {
                for r in &results {
                    println!("{}", r.display());
                }
            }
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
                        eprint!("input number (0-{}): ", results.len() - 1);
                        io::stderr().flush().unwrap();
                        io::stdin().read_line(&mut input).unwrap();
                        let input = input.trim();
                        match input.parse::<usize>() {
                            Ok(x) if x < results.len() => break x,
                            _ => eprintln!(
                                "please type a number inside the range 0 to {}",
                                results.len() - 1
                            ),
                        }
                    };

                    println!("{}", results[num].display());
                }
            }
        }
    }
    Ok(())
}

// read input arguments
// - direct serach
// - basename / nobasename
// - outputall or choose best or interactive
// - verbose mode for discovering what indexing is slow
// - numthreads or seqential
// - where to search

fn app() -> Result<(), Box<dyn Error>> {
    //TODO make query pipeable in stdin => path collecting only once => search reuse
    //TODO replace all expects and unwraps with error!
    //TODO use builder macro from usage
    //TODO use exit codes

    let options = parse_commandline_args();

    trace!("Arguments: {:?}", options);

    let paths = if options.use_cache {
        let mut cache_file = dirs::cache_dir().unwrap();
        cache_file.push("project-chooser.cache");
        //TODO make cache loading async and return mpsc::Receiver
        let cache = Cache::load(&cache_file).unwrap();

        if cache.entries.is_empty() {
            warn!("cache file seems to be empty");
        }
        cache.entries.iter().map(|&(_, ref e)| e.clone()).collect()
    } else {
        debug!("no cache");
        gather_projects(&options.root)
    };
    display_results(paths, options)
}

fn main() {
    process::exit(match app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            1
        }
    });
}
