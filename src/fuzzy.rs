use cuid::cuid2;
use serde::Serialize;
use std::{
    fs::{self, DirEntry, File},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};

#[derive(Debug, Serialize, Clone)]
pub struct Fuzz {
    /// id for fuzz
    id: String,

    /// full path of the file or dir
    path: PathBuf,

    /// name of the file or dir
    name: String,

    /// file size in bytes
    size: usize,

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

impl Fuzz {
    pub fn new() -> Fuzz {
        Fuzz {
            id: cuid2(),
            path: PathBuf::new(),
            name: String::new(),
            is_dir: false,
            is_file: false,
            is_expanded: false,
            direct_parent: PathBuf::new(),
            parents: Vec::new(),
            spacer: String::new(),
            children: 0,
            size: 0,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    pub fn spacer(&self) -> String {
        self.spacer.to_owned()
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn parents(&self) -> Vec<PathBuf> {
        self.parents.to_vec()
    }

    pub fn direct_parent(&self) -> PathBuf {
        self.direct_parent.to_owned()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn hsize(&self) -> String {
        let mut size = self.size;

        // convert in kb
        if size > 1000 {
            size /= 1000;

            // convert in mb
            if size > 1000 {
                size /= 1000;

                size.to_string() + " mb"
            } else {
                size.to_string() + " kb"
            }
        } else {
            size.to_string() + " bytes"
        }
    }

    fn set_parents(&mut self, parents: Vec<PathBuf>) -> &mut Self {
        self.parents = parents;
        self
    }

    fn set_spacer(&mut self, spacer: String) -> &mut Self {
        self.spacer = spacer;
        self
    }

    fn read_path(&mut self, path: PathBuf) {
        let meta = if let Ok(f) = File::open(&path) {
            if let Ok(meta) = f.metadata() {
                meta
            } else {
                return;
            }
        } else {
            return;
        };

        self.name = if let Some(name) = path.file_name() {
            name.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        };

        self.size = meta.size() as usize;
        self.path = path;
        self.is_dir = meta.is_dir();
        self.is_file = meta.is_file();
        self.is_expanded = false;

        if let Some(p) = self.path().parent() {
            self.direct_parent = p.to_owned();
        };
    }

    fn read_dir_entry(&mut self, entry: DirEntry) {
        let meta = if let Ok(meta) = entry.metadata() {
            meta
        } else {
            return;
        };

        self.size = meta.size() as usize;
        self.path = entry.path();
        self.name = entry.file_name().to_string_lossy().to_string();
        self.is_dir = entry.path().is_dir();
        self.is_file = entry.path().is_file();

        if let Some(p) = self.path().parent() {
            self.direct_parent = p.to_owned();
        };
    }

    fn create(&mut self, path: PathBuf) {
        if path.to_string_lossy().ends_with("/") {
            fs::create_dir(&path).unwrap();
        } else {
            File::create_new(&path).unwrap();
        };

        self.read_path(path);
    }

    fn remove(&mut self) {
        let path = &self.path;

        if self.is_file {
            let _ = fs::remove_file(path);
        }

        if self.is_dir {
            let _ = fs::remove_dir_all(path);
        }
    }

    fn rename(&mut self, name: String) {
        let old_path = self.path.to_owned();
        let new_path = self.path.parent().unwrap().join(&name);

        self.path = new_path.to_path_buf();
        self.name = name;

        let _ = fs::rename(old_path, new_path);
    }
}

#[derive(Debug)]
pub struct Fuzzy {
    base_path: PathBuf,
    fuzzies: Vec<Fuzz>,
}

impl Fuzzy {
    pub const TOP_SPACER: &str = " ";
    pub const NESTED_SPACER: &str = "|-";

    pub fn new(path: PathBuf) -> Self {
        let mut scanner = Scanner::new();

        // set default top level spacer
        scanner
            .set_spacer(Fuzzy::TOP_SPACER.to_string())
            .set_parents([path.to_owned()].to_vec());

        Fuzzy {
            base_path: path.to_owned(),
            fuzzies: scanner.scan(path),
        }
    }

    pub fn base_path(&self) -> String {
        self.base_path.to_string_lossy().to_string()
    }

    pub fn fuzzies(&self) -> &Vec<Fuzz> {
        &self.fuzzies
    }

    pub fn get_fuzzy(&self, idx: usize) -> Option<&Fuzz> {
        self.fuzzies.get(idx)
    }

    pub fn toggle_fuzzy(&mut self, idx: usize) {
        let fuzzy = if let Some(f) = self.fuzzies.get(idx) {
            f
        } else {
            return;
        };

        if fuzzy.is_expanded {
            self.collaspe_fuzzy(idx);
        } else {
            self.expand_fuzzy(idx);
        }
    }

    pub fn expand_fuzzy(&mut self, idx: usize) {
        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        // expand is only for dir
        if fuzzy.is_file {
            return;
        }

        let path = fuzzy.path.to_owned();

        // set spacer for children
        let mut spacer = fuzzy.spacer.to_string();
        spacer.push_str(Fuzzy::NESTED_SPACER.to_string().as_str());

        // set parents for children
        let mut parents = fuzzy.parents.to_vec();
        parents.push(path.to_owned());

        let child_fuzzies = Scanner::new()
            .set_spacer(spacer)
            .set_parents(parents)
            .scan(path.to_owned());

        fuzzy.children = child_fuzzies.len();
        fuzzy.is_expanded = true;

        let insert_to = idx + 1;
        self.fuzzies.splice(insert_to..insert_to, child_fuzzies);
    }

    pub fn collaspe_fuzzy(&mut self, idx: usize) {
        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        if fuzzy.is_file {
            return;
        }

        let path = fuzzy.path.to_owned();
        fuzzy.is_expanded = false;
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
        let insert_to = idx + 1;
        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        let path: PathBuf;
        let mut fuzz = Fuzz::new();

        if fuzzy.is_file {
            path = fuzzy.direct_parent.join(&name);

            fuzz.set_parents(fuzzy.parents.to_vec());
            fuzz.set_spacer(fuzzy.spacer.to_owned());
        } else {
            path = fuzzy.path.join(&name);
            let mut parents = fuzzy.parents.to_owned();
            parents.push(fuzzy.path.to_owned());

            fuzz.set_parents(parents);
            fuzz.set_spacer(fuzzy.spacer.to_owned() + Fuzzy::NESTED_SPACER);
        };

        fuzz.create(path);
        self.fuzzies.splice(insert_to..insert_to, [fuzz]);
    }

    pub fn remove_fuzzy(&mut self, idx: usize) {
        // because all the chidlren needs to be removed also if dir
        // is removed so collasping removes the fuzz entries
        // in the list
        //
        // another way is to remove the children one by one for the
        // selected fuzzy
        self.collaspe_fuzzy(idx);

        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        fuzzy.remove();
        self.fuzzies.remove(idx);
    }

    pub fn rename_fuzzy(&mut self, idx: usize, name: String) {
        // because all the chidlren needs to update there parents list if dir
        // name is updated
        self.collaspe_fuzzy(idx);

        let fuzzy = if let Some(f) = self.fuzzies.get_mut(idx) {
            f
        } else {
            return;
        };

        fuzzy.rename(name);
    }
}

struct Scanner {
    spacer: String,
    parents: Vec<PathBuf>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            spacer: String::new(),
            parents: Vec::new(),
        }
    }

    pub fn scan(&mut self, path: PathBuf) -> Vec<Fuzz> {
        let mut fuzzies = Vec::new();

        if let Ok(entries) = fs::read_dir(&path) {
            // this will give the only success entries
            for entry in entries.flatten() {
                let mut fuzz = Fuzz::new();

                fuzz.set_parents(self.parents.to_owned())
                    .set_spacer(self.spacer.to_owned())
                    .read_dir_entry(entry);

                fuzzies.push(fuzz);
            }
        }

        fuzzies
    }

    fn set_spacer(&mut self, s: String) -> &mut Self {
        self.spacer = s;
        self
    }

    fn set_parents(&mut self, p: Vec<PathBuf>) -> &mut Self {
        self.parents.append(&mut p.to_owned());
        self
    }
}
