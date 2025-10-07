use serde::{Deserialize, Serialize};

pub mod serialization;

use serialization::*;

mod version0;
pub use version0::*;

pub(crate) mod param;

mod version1;
pub use version1::*;

use crate::patching::param::Parameter;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatchOffsetType {
    Raw,
    Virtual,
}

pub trait HasPatches {
    fn add_patch(&mut self, patch: Patch);
    fn get_patches(&self) -> &Vec<Patch>;

    fn set_patches(&mut self, patch: Vec<Patch>) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Clone, Default, Debug)]
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
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchEntry {
    name: String,
    description: String,

    // Specified if another author made a specific patch in a patch list
    alt_author: Option<String>,

    parameters: Vec<Parameter>,

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
            parameters: Default::default(),
            patches,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Patch {
    #[serde(serialize_with = "se_u32_to_hex", deserialize_with = "de_hex_to_u32")]
    pub offset: u32,

    pub offset_type: PatchOffsetType,

    #[serde(serialize_with = "se_vu8_to_hex", deserialize_with = "de_hex_to_vu8")]
    pub replacement_bytes: Vec<u8>,

    #[serde(
        serialize_with = "se_ovu8_to_hex",
        deserialize_with = "de_hex_to_ovu8",
        skip_serializing_if = "Option::is_none",
        default = "get_none"
    )]
    pub original_bytes: Option<Vec<u8>>,
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
