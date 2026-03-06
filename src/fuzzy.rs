use serde::Serialize;
use std::{fs, path::PathBuf};

use crate::constants;

#[derive(Debug, Serialize)]
pub struct Fuzz {
    /// full path of the file or dir
    path: PathBuf,

    /// name of the file or dir
    name: String,

    /// if path is dir or not
    is_dir: bool,

    /// if path is dir or not
    is_file: bool,

    /// if path is expanded or not
    is_expanded: bool,

    /// path of direct parent dir
    direct_parent: PathBuf,

    /// all the parents
    parents: Vec<PathBuf>,

    /// spacer to render
    spacer: String,

    /// children of fuzz
    children: usize,
}

#[derive(Debug)]
pub struct Fuzzy {
    base_path: PathBuf,
    fuzzies: Vec<Fuzz>,
}

impl Fuzzy {
    pub fn new(path: PathBuf) -> Fuzzy {
        Fuzzy {
            base_path: path.to_owned(),
            fuzzies: Scanner::new().scan(path),
        }
    }

    pub fn fuzzies(&self) -> &Vec<Fuzz> {
        self.fuzzies.as_ref()
    }

    pub fn expand_fuzzy(&mut self, idx: usize) {
        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        let path = fuzzy.path.to_owned();
        let child_fuzzies = Scanner::new()
            .spacer(constants::NESTED_SPACER.to_string())
            .parents(fuzzy.parents.to_owned())
            .scan(path.to_owned());

        fuzzy.is_expanded = true;
        fuzzy.children = child_fuzzies.len();

        let insert_to = idx + 1;
        self.fuzzies.splice(insert_to..insert_to, child_fuzzies);
    }

    pub fn collaspe_fuzzy(&mut self, idx: usize) {
        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        let path = fuzzy.path.to_owned();
        fuzzy.is_expanded = true;
        fuzzy.children = 0;

        let mut remove_idxs: Vec<usize> = Vec::new();
        for (i, fuzzy) in self.fuzzies.iter().enumerate() {
            if fuzzy.parents.iter().any(|p| p == &path) {
                remove_idxs.push(i);
            }
        }

        if !remove_idxs.is_empty() {
            let remove_idx_from = remove_idxs[0];
            let remove_idx_to = remove_idxs[remove_idxs.len() - 1] + 1;

            self.fuzzies.drain(remove_idx_from..remove_idx_to);
        }
    }

    pub fn create_fuzzy(&mut self, idx: usize, name: String) {
        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        let where_to = if fuzzy.is_file {
            fuzzy.direct_parent.to_owned()
        } else {
            fuzzy.path.to_owned()
        };
        let full_path = where_to.join(&name);

        // if ends with "/" create dir else file
        if name.ends_with("/") {
            fs::create_dir_all(&full_path).expect("unable to create dirs");
        } else {
            fs::File::create_new(&full_path)
                .expect("unable to create file or intermidiate dir does not exits");
        }

        let insert_to = idx;
        let mut file_name = name.to_owned();
        file_name.pop();

        self.fuzzies.splice(
            insert_to..insert_to,
            vec![Fuzz {
                path: full_path.to_owned(),
                name: file_name,
                is_file: where_to.is_file(),
                is_dir: where_to.is_dir(),
                direct_parent: where_to.to_owned(),
                parents: Vec::new(),
                is_expanded: false,
                children: 0,
                spacer: String::from(" "),
            }],
        );
    }
}

struct Scanner {
    spacer: String,
    parents: Vec<PathBuf>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            spacer: String::from(" "),
            parents: Vec::new(),
        }
    }

    pub fn scan(&mut self, path: PathBuf) -> Vec<Fuzz> {
        let mut fuzzies = Vec::new();

        if let Ok(entries) = fs::read_dir(&path) {
            // this will give the only success entries
            for entry in entries.flatten() {
                let mut fuzz = Fuzz {
                    path: entry.path(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_file: false,
                    is_dir: false,
                    direct_parent: path.to_owned(),
                    is_expanded: false,
                    spacer: self.spacer.to_owned(),
                    children: 0,
                    parents: self.parents.to_owned(),
                };

                if let Ok(meta) = entry.metadata() {
                    fuzz.is_dir = meta.is_dir();
                    fuzz.is_file = meta.is_file();
                }

                fuzz.parents.push(path.to_owned());
                fuzzies.push(fuzz);
            }
        }

        fuzzies
    }

    fn spacer(&mut self, s: String) -> &mut Self {
        self.spacer = s;
        self
    }

    fn parents(&mut self, p: Vec<PathBuf>) -> &mut Self {
        self.parents.append(&mut p.to_owned());
        self
    }
}
