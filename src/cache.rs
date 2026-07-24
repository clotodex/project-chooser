use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::path::{Path, PathBuf};

//line in file: (tab separated)
//50	/home/...
type Frequency = u64; //TODO change to usize for performance?
type Entry = (Frequency, PathBuf); //(frequency, path)

//TODO write tests!
//TODO write trace and debug messages and info
pub struct Cache {
    //TODO maybe dont save filepath so load and save can use seperate ones
    pub file: PathBuf, //TODO or use box => maybe better
    pub entries: Vec<Entry>,
}

impl Cache {
    pub fn load(file: &Path) -> Result<Self> {
        let entries = {
            //let file = File::create(&file)?;
            (File::open(&file)
                .map(|file| {
                    let reader = BufReader::new(file);
                    let lines_iter = reader.lines().map(std::result::Result::unwrap);
                    Ok(lines_iter
                        .map(|line| {
                            //TODO ignore empty lines
                            let mut it = line.trim().splitn(2, |c| c == ' ' || c == '\t');
                            //TODO trim might not be necessary since pathbuf might trim
                            (
                                it.next().unwrap().parse::<Frequency>().unwrap(),
                                PathBuf::from(it.next().unwrap().trim()),
                            )
                        })
                        .collect())
                })
                .unwrap_or_else(|_| {
                    //TODO match error for not exists
                    File::create(&file)?;
                    Ok(vec![])
                }) as Result<Vec<Entry>>)?
        };
        Ok(Cache {
            file: file.to_path_buf(),
            entries,
        })
    }

    pub fn new(file: &Path) -> Self {
        Cache {
            file: file.to_path_buf(),
            entries: vec![],
        }
    }

    pub fn select(&mut self, selection: &PathBuf) {
        let item = if let Some(index) = self.entries.iter().position(|e| &e.1 == selection) {
            let mut item = self.entries.remove(index);
            item.0 += 1;
            item
        } else {
            println!("not found");
            //TODO maybe should throw error since the path most likely does not exist anymore
            (1, selection.to_path_buf())
        };
        //most recent item => start of the list
        self.entries.insert(0, item);
    }

    pub fn update(&mut self, list: &[PathBuf]) {
        let new: Vec<Entry> = self
            .entries
            .iter()
            .filter(|&&(_, ref e)| list.contains(e))
            .map(|&(ref f, ref e)| (*f, e.to_path_buf()))
            .collect();
        let iter: Vec<Entry> = list
            .iter()
            .filter(|&e| !new.iter().any(|&(_, ref n)| n == e))
            .map(|e| (0, e.to_path_buf()))
            .collect();
        self.entries = new.into_iter().chain(iter).collect();
        //TODO return when changed
    }

    //TODO return if anything changed
    /*pub fn update_broken(&mut self, list: &[PathBuf]) {
        //TODO remove from entries what is not contained in list
        //TODO add to back of entries what did not exist yet

        let mut removals: Vec<usize> = vec![];

        //O(n log n)
        let mut alphabetical_indices_of_entries: Vec<usize> = (0..self.entries.len()).collect();
        alphabetical_indices_of_entries.sort_by(|&a, &b| self.entries[a].1.cmp(&self.entries[b].1));

        let mut it = alphabetical_indices_of_entries.iter();
        let mut ind_e_opt = it.next();
        let mut it_list = list.iter();
        let mut elem_opt = it_list.next();
        loop {
            if let Some(elem) = elem_opt {
            if let Some(&ind_e) = ind_e_opt {
                println!("Comparing '{:?}' - '{:?}'", self.entries[ind_e].1, elem);
                match self.entries[ind_e].1.cmp(&elem) {
                    // cached value does not exist anymore => mark for removal
                    Ordering::Less => {
                        println!("to be removed: {}", self.entries[ind_e].1.display());
                        removals.push(ind_e);
                        ind_e_opt = it.next();
                        break;
                    },
                    // cached value exists => no change
                    Ordering::Equal => {
                        println!("no change: {}", self.entries[ind_e].1.display());
                        ind_e_opt = it.next();
                        elem_opt = it_list.next();
                    },
                    // update value is new => add to entries
                    Ordering::Greater => {
                        println!("new item: {}", elem.display());
                        self.entries.push((0, elem.to_path_buf()));
                        elem_opt = it_list.next();
                    }
                }
            } else {
                //TODO integrate this via an OR statement into the Ordering match
                println!("adding from current comparison: {}", elem.display());
                self.entries.push((0, elem.to_path_buf()));
                break;
            }
            } else {
                if let Some(&ind_e) = ind_e_opt {
                    println!("removing from comparison: {}", self.entries[ind_e].1.display());
                    removals.push(ind_e);
                }
                break;
            }
        }

        // add leftover new ones
        for elem in it_list {
            println!("new leftover: {}", elem.display());
            self.entries.push((0, elem.to_path_buf()));
        }

        // leftover removals
        let removals = removals.iter().chain(it);

        // removals are in sorted order
        // thus it is safe to remove elements and always substract it from the following indicies
        let mut sub = 0;
        for rem in removals {
            //TODO crash in next line - not sorted because of chain?
            println!("actually removing: {}", self.entries[rem - sub].1.display());
            self.entries.remove(rem - sub);
            sub += 1;
        }


        //retainall
        //addall
    }*/

    fn normalize(&mut self) {
        //TODO get lowest frequency => set to 0
        //substract that frequency from every other entry
        //=> prevents frequency overflow
        error!("normalizing is not implemented yet")
    }

    pub fn save(&mut self) -> Result<()> {
        self.normalize();
        let mut buffer = File::create(&self.file)?;
        for &(freq, ref path) in &self.entries {
            writeln!(buffer, "{}\t{}", freq, path.display())?;
        }
        Ok(())
    }
}
