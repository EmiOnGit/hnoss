use crate::{
    MainCamera,
    asset_loading::LoadResource,
    editor::spawn_tile,
    io::{self, SaveFile},
    map,
};
use bevy::{prelude::*, window::PrimaryWindow};

/// Tilesize of a single tile in the sheet in pixel.
/// Width and Height are both the same
pub const TILESIZE: i32 = 16;
pub const TEXTURE_PATH: &str = "textures.png";

pub fn plugin(app: &mut App) {
    app.init_resource::<Textures>()
        .init_asset_loader::<io::SaveFileAssetLoader>()
        .init_asset::<io::SaveFile>()
        .init_resource::<MousePosition>()
        .add_systems(Update, (update_mouse_position, load_level))
        .load_resource::<Textures>();
}
#[derive(
    Component, Default, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize, Hash,
)]
pub enum LayerType {
    #[default]
    Bg,
    Fg,
}
impl LayerType {
    pub fn name(&self) -> &'static str {
        (*self).into()
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Bg,
            1 => Self::Fg,
            _ => {
                error!("Tried to construct layertype {v}. There are not that many variants.");
                Self::Fg
            }
        }
    }
    pub fn next(&self) -> Self {
        LayerType::from_u8((*self as u8 + 1) % 2)
    }
    pub fn z(&self) -> f32 {
        match self {
            LayerType::Bg => 0.,
            LayerType::Fg => 1.,
        }
    }
}
impl Into<&'static str> for LayerType {
    fn into(self) -> &'static str {
        match self {
            LayerType::Bg => "Background",
            LayerType::Fg => "Foreground",
        }
    }
}
pub struct TexturePack {
    tex_path: &'static str,
    tex: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}
#[derive(Resource, Asset, Reflect)]
pub struct Textures {
    pub textures: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}
impl FromWorld for Textures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let textures = asset_server.load(TEXTURE_PATH);
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(TILESIZE as u32), 4, 4, None, None);
        let layout = texture_atlas_layouts.add(layout);
        Textures { textures, layout }
    }
}
#[derive(Resource, Default)]
pub struct MousePosition {
    pub world_position: Vec2,
}
impl MousePosition {
    /// Converts the world position onto the left bottom corner of the tile grid
    pub fn to_tile_grid_lb(&self) -> IVec2 {
        convert_to_tile_grid(self.world_position)
    }
    /// Converts the world position onto the center of the corresponding tile grid cell
    pub fn to_tile_grid_center(&self) -> IVec2 {
        self.to_tile_grid_lb() + IVec2::new(TILESIZE, TILESIZE) / 2
    }
}
/// Converts the world position onto the left bottom corner of the tile grid
pub fn convert_to_tile_grid(position: Vec2) -> IVec2 {
    IVec2::new(
        (position.x as i32).div_euclid(TILESIZE) * TILESIZE,
        (position.y as i32).div_euclid(TILESIZE) * TILESIZE,
    )
}
/// Updates the world position of the cursor every frame for other systems to use
fn update_mouse_position(
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_position: ResMut<MousePosition>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((cam, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(position) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };
    mouse_position.world_position = position;
}
fn load_level(
    mut events: EventReader<AssetEvent<SaveFile>>,
    mut commands: Commands,
    save_files: Res<Assets<SaveFile>>,
    tiles_q: Query<(Entity, &Sprite, &Transform, &LayerType)>,
    textures: Res<map::Textures>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                for (e, _, _, _) in &tiles_q {
                    commands.entity(e).despawn();
                }
                let level = save_files.get(*id).unwrap();
                for (layer_type, tiles) in &level.layers {
                    for tile in &tiles.tiles {
                        commands.spawn(spawn_tile(tile.pos, &textures, tile.index, *layer_type));
                    }
                }
            }
            _ => {}
        }
    }
}
