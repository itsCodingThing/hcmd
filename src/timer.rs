use cuid::cuid2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Timer {
    id: String,
    name: String,
    duration: u64,
}

pub fn new(name: String, duration: u64) -> Timer {
    Timer {
        id: cuid2(),
        name,
        duration,
    }
}

impl Timer {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }
}
