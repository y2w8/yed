use std::{fs::{File, OpenOptions},io::Write, sync::Mutex};

use anyhow::Ok;

pub struct Logger {
    file: Mutex<File>
}

impl Logger {
    pub fn new(file: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file)
            .expect("Unable to open log file");

        Logger { 
            file: Mutex::new(file)
        }
    }

    pub fn log(&self, message: &str) -> anyhow::Result<()> {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", message)?;
        Ok(())
    }
}

