# Project Chooser

#### A tool for searching in your project folder

#### By **clotodex**

## Features - How it works

When you have a projects folder where you have many projects or cloned repositories etc it quickly gets difficult to navigate the paths to get to that specific project you started two weeks ago.  
This tool helps you by identifying all your projects and offering you search functionality.

You can place one of the following files into your directory to mark it as a project:
- ```.project``` or ```.git``` - this will also exclude the directory from deeper indexing
- ```.groupproject``` - this will tell the tool to also look fro other projects inside this directory

An example directory structure
    
    projects
    ├── cloned_project/
    │   └── .git/
    ├── parentProject/
    │   ├── binary/
    │   │   └── .git/
    │   ├── library/
    │   │   └── .git/
    │   └── .groupproject
    ├── projectA/
    │   └── .project
    └── projectB/
        ├── data/
        └── .project

This wil find the following projects: ```cloned_project```, ```parent_project```, ```binary```, ```library```, ```projectA```, ```projectB```  
The ```.project``` file in projectB also prevents the tool from searching through ```data/```.

Because of this special indexing this tool is actually faster than GNU find (for this sepecific task).

For specific features please have a look at the [Usage section](#usage)

<div id="usage" />
## Usage

You can call ```project-chooser --help``` at any point to get a full list of options.

Main usage:

    project-chooser [OPTIONS] [query]

Options:

- ```-b, --basename     Search in path basenames instead of the whole path```
- ```-c, --cache        Cache all project paths for future use```
- ```-h, --help         Prints help information```
- ```-q, --quiet        Silence all output```
- ```-V, --version      Prints version information```
- ```-v                 Increase message verbosity (-vvvv = Trace-level)```
- ```-j, --jobs <jobs>  Amount of threads to use (default: None)```
- ```-m, --mode <mode>  How to handle multiple results (default: INTERACTIVE) [possible values: ALL, FIRST, INTERACTIVE]```
- ```-p, --path <path>  Set root path for locating projects (default: "~/projects")```

ARGS:

- ```query```: A string to search for - if omitted dmenu will be opened where you can search in all your projects interactively 

### Aliases

This tool outputs the path to the selected project.  
Since I navigate my filesystem via the terminal on linux I can use aliases to call the tool and directly change the directory.  
I you want to do this too, please have a look at the [aliases file](/aliases.sh)

You can also use the output to call any file-browser you like with the path.
For windows for example you could write a .bat file calling the tool and starting explorer.exe with the path as an arg.

I can add all integration-scripts to this repository, just create an issue with the script.

## Future ideas and TODOs

- proper return codes (no selection or error produces error code)
- cache functionality (display cache and index projects in background, updating the cache)
- save cache in "most recently" order and have a selection count => should add option to sort alphabetically as well
- option for a dry-run only updating the cache (could be run via cronjob)
- metainfo in cachefile (e.g. timestamp of last cache update => can decide based on time difference if another indexing is necessary)
- multi-threading (is I/O the bottleneck or can I/O be done in multiple threads?)
- make graphical selection a compile time feature
- have other graphical options (gtk, qt, windows)
- allow .git AND .groupproject
- place performance results here
- show an example project structure or even have a small gif showing the tool in action

## Installation

The only soft dependency is the tool [dmenu](https://tools.suckless.org/dmenu/)  
The tool is only used if no query is given

### Binary

TODO

### Building from source

1. install rust as shown [here](https://www.rust-lang.org/install.html)
2. clone this repository
3. run ```cargo build --release```
4. the binary is saved to <clone-dir>/target/release/project-chooser

## License

see [LICENSE](/LICENSE) file

Contact me if you need a different license
