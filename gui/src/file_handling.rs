use serde::{Deserialize, Serialize};
use serde_json::Result;

// TODO: Make this actually shared between the 2 apps
pub struct GamePatch {
    name: String,
    offset: u32,
    offset_type: GamePatchOffsetType,
    replacement_bytes: Vec<u8>,
    original_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GamePatchOffsetType {
    Raw,
    Virtual,
}

#[derive(Serialize, Deserialize)]
pub struct PatchSet {
    xbpatchset_schema: u32,
    name: String,
    author: String,
    version_major: u8,
    version_minor: u8,
    game_title: String,
    patch_list: Vec<GamePatch>,
}
