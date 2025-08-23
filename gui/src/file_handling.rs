use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use xbpatch_core::patching::PatchEntry;

#[derive(Serialize, Deserialize, Clone)]
pub struct PatchSet {
    pub xbpatchset_schema: u32,
    pub name: String,
    pub author: String,
    pub version_major: u8,
    pub version_minor: u8,
    pub game_title: String,
    pub entries: Vec<PatchEntry>,
}

impl PatchSet {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub struct LoadedPatchSet {
    file: LiveFile<PatchSet>,
    patch_set: PatchSet,
    // Array of indices showing which patch entries are enabled
    enabled_entries: Vec<u8>,
}

impl LoadedPatchSet {
    pub fn new(path: &PathBuf) -> Result<Self, std::io::Error> {
        let file = LiveFile::<PatchSet>::from_existing(path)?;
        let patch_set: PatchSet = file.data().clone();

        Ok(LoadedPatchSet {
            file,
            patch_set,
            enabled_entries: Vec::new(),
        })
    }

    pub fn path(&self) -> &PathBuf {
        self.file.path()
    }
    pub fn data(&self) -> &PatchSet {
        &self.patch_set
    }
}

pub struct LiveFile<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    file: File,
    path: PathBuf,
    backup_path: PathBuf,
    data: T,
}

impl<T> LiveFile<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn from_new(path: &PathBuf, data: T) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().read(true).write(true).open(&path)?;

        let mut backup_path = path.clone();
        backup_path.set_extension("json.bak");

        Ok(LiveFile {
            file,
            path: path.clone(),
            backup_path,
            data,
        })
    }

    pub fn from_existing(path: &PathBuf) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().read(true).write(true).open(&path)?;

        let data: T = serde_json::from_reader(&file)?;

        let mut backup_path = path.clone();
        backup_path.set_extension("json.bak");

        Ok(LiveFile {
            file,
            path: path.clone(),
            backup_path,
            data,
        })
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        // Clear the file
        self.file.seek(SeekFrom::Start(0))?;
        self.file.set_len(0)?;

        std::fs::copy(&self.path, &self.backup_path)?;
        serde_json::to_writer_pretty(&self.file, &self.data)?;
        std::fs::remove_file(&self.backup_path)?;

        Ok(())
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn update<F>(&mut self, f: F) -> Result<(), std::io::Error>
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.data);
        self.save()?;

        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
