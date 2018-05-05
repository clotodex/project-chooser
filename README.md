# Project Chooser

#### A tool for searching in your project folder

#### By **clotodex**

## Features

When you have a projects folder where you have many projects or cloned repositories etc it quickly gets difficult to navigate the paths to get to that specific project you started two weeks ago.  
This tool helps you by identifying all your projects and offering you search functionality.

See [Usage](#usage)

<div id="usage" />
## Usage

You can call ```project-chooser --help``` at any point to get a full list of options.

Main usage:

    project-chooser [OPTIONS] [query]

Options:

- ```-b, --basename   Search in path basenames instead of the whole path```
- ```-c, --cache      Cache all project paths for future use```
- ```-h, --help       Prints help information```
- ```-p, --path       Set root path for locating projects (default: "~/projects")```
- ```-q, --quiet      Silence all output```
- ```-V, --version    Prints version information```
- ```-v               Increase message verbosity (-vvvv = Trace-level)```
- ```-j <jobs>        Amount of threads to use (default: None)```
- ```-m <mode>        How to handle multiple results (default: INTERACTIVE) [possible values: ALL, FIRST, INTERACTIVE]```

ARGS:

- ```query```: A string to search for - if omitted dmenu will be opened where you can search in all your projects interactively 

## Future ideas and TODOs

- proper return codes (no selection or error produces error code)
- cache functionality (display cache and index projects in background, updating the cache)
- save cache in "most recently" order and have a selection count => should add option to sort alphabetically as well
- multi-threading (is I/O the bottleneck or can I/O be done in multiple threads?)
- make graphical selection a compile time feature
- have other graphical options (gtk, qt, windows)

## Installation

The only soft dependency is the tool [dmenu](https://tools.suckless.org/dmenu/)  
The tool is only used if no query is given

### Binary

TODO

### Building from source

1) install rust as shown [here](https://www.rust-lang.org/install.html)
2) clone this repository
3) run ```cargo build --release```
4) the binary is saved to <clone-dir>/target/release/project-chooser

## License

see [LICENSE](/LICENSE) file

Contact me if you need a different license
