use crate::map::LayerType;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use ron::ser::PrettyConfig;
use std::collections::HashMap;
use thiserror::Error;

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

#[derive(Asset, TypePath, serde::Serialize, serde::Deserialize, Default)]
pub struct SaveFile {
    pub layers: HashMap<LayerType, Layer>,
}

#[derive(Default)]
pub struct SaveFileAssetLoader;

#[derive(Debug, Error)]
pub enum SaveFileAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}
impl AssetLoader for SaveFileAssetLoader {
    type Asset = SaveFile;
    type Settings = ();
    type Error = SaveFileAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<SaveFile>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
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
