use serde::{Deserialize, Serialize};

use crate::patching::{PatchOffsetType, param::Endiannness};

use super::serialization::*;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PatchSetV1 {
    pub xbpatchset_schema: u32,
    pub name: String,
    pub author: String,
    pub version_major: u8,
    pub version_minor: u8,
    pub game_title: String,
    pub entries: Vec<PatchEntryV1>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchEntryV1 {
    name: String,
    description: String,

    // Specified if another author made a specific patch in a patch list
    alt_author: Option<String>,

    parameters: Vec<SerializedParameter>,

    patches: Vec<PatchV1>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatchReplacement {
    #[serde(serialize_with = "se_vu8_to_hex", deserialize_with = "de_hex_to_vu8")]
    Bytes(Vec<u8>),
    Parameter(usize),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchV1 {
    #[serde(serialize_with = "se_u32_to_hex", deserialize_with = "de_hex_to_u32")]
    pub offset: u32,

    pub offset_type: PatchOffsetType,

    // #[serde(serialize_with = "se_vu8_to_hex", deserialize_with = "de_hex_to_vu8")]
    // pub replacement_bytes: Vec<u8>,
    pub replacement: PatchReplacement,
    /*
    #[serde(
        serialize_with = "se_ovu8_to_hex",
        deserialize_with = "de_hex_to_ovu8",
        skip_serializing_if = "Option::is_none",
        default = "get_none"
    )]
    pub original_bytes: Option<Vec<u8>>,
    */
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializedParameter {
    name: String,

    #[serde(rename = "type")]
    type_name: String,

    endianness: Endiannness,

    default: Option<String>,
    min: Option<String>,
    max: Option<String>,

    value: String,
}
