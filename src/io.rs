use crate::map::LayerType;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use ron::ser::PrettyConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
pub fn select_file() -> Option<PathBuf> {
    None
}
#[cfg(not(target_arch = "wasm32"))]
pub fn select_file() -> Option<PathBuf> {
    let fd = rfd::FileDialog::new()
        .add_filter("level", &["ron"])
        .set_directory("assets/");
    fd.pick_file().map(|path| {
        path.iter()
            .skip_while(|p| p.to_str().unwrap() != "assets")
            .skip(1)
            .collect()
    })
}
#[cfg(target_arch = "wasm32")]
pub fn save(file: &SaveFile) {
    let file_string =
        ron::ser::to_string_pretty(file, PrettyConfig::new().compact_structs(true)).unwrap();
    println!("{file_string}");
}
#[cfg(not(target_arch = "wasm32"))]
pub fn save(file: &SaveFile) {
    use rfd::FileDialog;
    let file_string =
        ron::ser::to_string_pretty(file, PrettyConfig::new().compact_structs(true)).unwrap();

    let path = FileDialog::new()
        .add_filter("ron", &["ron"])
        .set_directory("assets")
        .set_file_name("level.ron")
        .save_file();

    if let Some(path) = path {
        std::fs::write(path, file_string).unwrap();
        info!("Saved successfully");
    } else {
        info!("Save was aborted since no file was selected");
    }
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
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Tile {
    /// grid position
    pub pos: IVec2,
    /// texture atlas index
    pub index: usize,
}
