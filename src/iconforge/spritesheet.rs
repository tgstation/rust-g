use dashmap::DashMap;
use serde::Serialize;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash64;

#[derive(Serialize)]
pub struct SpritesheetResult {
    pub sizes: Vec<String>,
    pub sprites: DashMap<String, SpritesheetEntry, BuildHasherDefault<XxHash64>>,
    pub dmi_hashes: DashMap<String, String>,
    pub sprites_hash: String,
    pub error: String,
}

#[derive(Serialize, Clone)]
pub struct SpritesheetEntry {
    pub size_id: String,
    pub position: u32,
}
