use std::{collections::HashMap, fs::File};

use bevy::math::IVec2;
use ron::ser::PrettyConfig;

use crate::map::LayerType;
pub fn save(file: &SaveFile, filename: &str) {
    let file_string = ron::ser::to_string_pretty(
        file,
        PrettyConfig::new()
            // .compact_arrays(true)
            .compact_structs(true),
    )
    .unwrap();
    std::fs::write(
        String::from("assets/level/") + filename + ".ron",
        file_string,
    )
    .unwrap();
}
pub fn load(filename: &str) -> anyhow::Result<SaveFile> {
    let path = String::from("assets/level/") + filename + ".ron";
    let file = std::io::BufReader::new(File::open(&path)?);
    let res = ron::de::from_reader(file)?;
    Ok(res)
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SaveFile {
    pub layers: HashMap<LayerType, Layer>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub tiles: Vec<Tile>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tile {
    /// grid position
    pub pos: IVec2,
    /// texture atlas index
    pub index: usize,
}
