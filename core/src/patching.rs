use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PatchOffsetType {
    Raw,
    Virtual,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patch {
    pub offset: u32,
    pub offset_type: PatchOffsetType,
    pub replacement_bytes: Vec<u8>,
    pub original_bytes: Option<Vec<u8>>,
}

pub trait HasPatches {
    fn add_patch(&mut self, patch: Patch);
    fn get_patches(&self) -> &Vec<Patch>;

    fn set_patches(&mut self, patch: Vec<Patch>) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchEntry {
    name: String,
    description: String,

    // Specified if another author made a specific patch in a patch list
    alt_author: Option<String>,

    patches: Vec<Patch>,
}

impl PatchEntry {
    pub fn new(
        name: String,
        description: String,
        alt_author: Option<String>,
        patches: Vec<Patch>,
    ) -> Self {
        PatchEntry {
            name,
            description,
            alt_author,
            patches,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl HasPatches for PatchEntry {
    fn get_patches(&self) -> &Vec<Patch> {
        &self.patches
    }

    fn add_patch(&mut self, patch: Patch) {
        self.patches.push(patch);
    }

    fn set_patches(&mut self, patch: Vec<Patch>) -> Result<(), Box<dyn std::error::Error>> {
        self.patches = patch;
        Ok(())
    }
}
