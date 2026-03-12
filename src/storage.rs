use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::Write,
};

use crate::timer::{self, Timer};

#[derive(Serialize, Deserialize, Debug)]
pub struct Storage {
    app_name: String,
    timers: Vec<Timer>,
}

pub fn create() -> Result<Storage> {
    let mut temp_file_path = env::temp_dir();
    temp_file_path.push("rusty-pomodoro/db.json");

    if let Some(parent) = temp_file_path.as_path().parent() {
        fs::create_dir_all(parent).expect("unable to create folders");
    }

    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(temp_file_path)
        .expect("unable to create file");

    let storage: Storage = serde_json::from_reader(file).unwrap_or_else(|err| {
        eprintln!("{err}");

        Storage {
            app_name: "rusty-pomodoro".to_string(),
            timers: Vec::new(),
        }
    });

    Ok(storage)
}

impl Storage {
    pub fn add(&mut self, name: String, duration: u64) {
        let t = timer::new(name, duration);

        self.timers.push(t);
        self.save().expect("unable to save file");
    }

    pub fn timers(&self) -> &Vec<Timer> {
        self.timers.as_ref()
    }

    pub fn remove_by_name(&mut self, name: String) {
        if !self.timers.is_empty() {
            let mut remove_idx = 0;

            for (i, timer) in self.timers.iter().enumerate() {
                if timer.name() == name.as_str() {
                    remove_idx = i;
                }
            }

            if remove_idx < self.timers.len(){
                self.timers.remove(remove_idx);
                self.save().expect("unable to save file");
            }
        }
    }

    pub fn remove_all(&mut self) {
        self.timers = Vec::new();
        self.save().expect("unable to save file");
    }

    fn save(&self) -> Result<()> {
        let result = serde_json::to_string(self).expect("unable to save file");
        let mut temp_file_path = env::temp_dir();
        temp_file_path.push("rusty-pomodoro/db.json");

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(temp_file_path)
            .expect("unable to read file");

        file.write_all(result.as_bytes())?;

        Ok(())
    }
}
