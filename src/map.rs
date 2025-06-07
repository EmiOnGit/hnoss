use std::collections::HashMap;

use crate::{
    MainCamera,
    asset_loading::LoadResource,
    editor::{RemoveOnLevelSwap, spawn_tile},
    entity::{self, OnSpawnTrigger, Player, Rule},
    io::{self, SaveFile},
    map,
    movement::DASH_RADIUS,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::tiles::TileStorage;

/// Tilesize of a single tile in the sheet in pixel.
/// Width and Height are both the same
pub const TILESIZE: i32 = 16;
pub const PLAYERSIZE: UVec2 = UVec2::new(18, 26);
pub const ENEMYSIZE: UVec2 = UVec2::new(18, 26);
pub const MAIN_TEXTURE_PATH: &str = "textures.png";
pub const ENTITY_TEXTURE_PATH: &str = "entities.png";
pub const FIRE_TEXTURE_PATH: &str = "fire.png";
pub const PLAYER_TEXTURE_PATH: &str = "char.png";
pub const ENEMIES_TEXTURE_PATH: &str = "enemies.png";

pub fn plugin(app: &mut App) {
    app.init_resource::<Textures>()
        .add_plugins(entity::plugin)
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
    Bg,
    Fg,
    #[default]
    Entities,
}
impl LayerType {
    pub fn name(&self) -> &'static str {
        (*self).into()
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Bg,
            1 => Self::Fg,
            2 => Self::Entities,
            _ => {
                error!("Tried to construct layertype {v}. There are not that many variants.");
                Self::Fg
            }
        }
    }
    pub fn next(&self) -> Self {
        LayerType::from_u8((*self as u8 + 1) % 3)
    }
    pub fn z(&self) -> f32 {
        match self {
            LayerType::Bg => 0.,
            LayerType::Fg => 1.,
            LayerType::Entities => 2.,
        }
    }
}
impl From<LayerType> for &'static str {
    fn from(val: LayerType) -> Self {
        match val {
            LayerType::Bg => "Background",
            LayerType::Fg => "Foreground",
            LayerType::Entities => "Entities",
        }
    }
}

#[derive(Reflect)]
pub struct TexturePack {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub rules: Vec<Rule>,
}
#[derive(Resource, Asset, TypePath)]
pub struct Textures {
    pub pack: HashMap<LayerType, TexturePack>,
    pub player: TexturePack,
    pub enemy: TexturePack,
    pub fire: TexturePack,
}
impl FromWorld for Textures {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let main_layout =
            TextureAtlasLayout::from_grid(UVec2::splat(TILESIZE as u32), 4, 4, None, None);
        let entity_layout =
            TextureAtlasLayout::from_grid(UVec2::splat(TILESIZE as u32), 8, 4, None, None);
        let fire_layout =
            TextureAtlasLayout::from_grid(UVec2::splat(TILESIZE as u32), 8, 4, None, None);
        let player_layout = TextureAtlasLayout::from_grid(PLAYERSIZE, 6, 3, None, None);
        let enemy_layout = TextureAtlasLayout::from_grid(ENEMYSIZE, 6, 3, None, None);
        let main_layout = texture_atlas_layouts.add(main_layout);
        let entity_layout = texture_atlas_layouts.add(entity_layout);
        let fire_layout = texture_atlas_layouts.add(fire_layout);
        let player_layout = texture_atlas_layouts.add(player_layout);
        let enemy_layout = texture_atlas_layouts.add(enemy_layout);
        let asset_server = world.resource::<AssetServer>();
        let mut map = HashMap::new();
        for layer in &[LayerType::Bg, LayerType::Fg, LayerType::Entities] {
            match layer {
                LayerType::Bg | LayerType::Fg => {
                    let main_textures = asset_server.load(MAIN_TEXTURE_PATH);
                    map.insert(
                        *layer,
                        TexturePack {
                            texture: main_textures,
                            layout: main_layout.clone(),
                            rules: vec![Rule::new(0, OnSpawnTrigger::Collider)],
                        },
                    );
                }
                LayerType::Entities => {
                    let texture = asset_server.load(ENTITY_TEXTURE_PATH);
                    map.insert(
                        *layer,
                        TexturePack {
                            texture,
                            layout: entity_layout.clone(),
                            rules: vec![
                                Rule::new(0, OnSpawnTrigger::Tower),
                                Rule::new(1, OnSpawnTrigger::Player),
                                Rule::new(2, OnSpawnTrigger::Enemy),
                            ],
                        },
                    );
                }
            }
        }
        let player = TexturePack {
            texture: asset_server.load(PLAYER_TEXTURE_PATH),
            layout: player_layout,
            rules: Vec::default(),
        };
        let enemy = TexturePack {
            texture: asset_server.load(ENEMIES_TEXTURE_PATH),
            layout: enemy_layout,
            rules: Vec::default(),
        };
        let fire = TexturePack {
            texture: asset_server.load(FIRE_TEXTURE_PATH),
            layout: fire_layout,
            rules: Vec::default(),
        };

        Textures {
            pack: map,
            player,
            enemy,
            fire,
        }
    }
}
#[derive(Resource, Default)]
pub struct MousePosition {
    pub world_position: Vec2,
    /// The point that can be used to find nearby enemies to dash to.
    /// Not the maximum dash distance
    pub dash_point: Vec2,
}
impl MousePosition {
    /// Converts the world position onto the left bottom corner of the tile grid
    pub fn to_tile_grid_lb(&self) -> IVec2 {
        convert_to_tile_grid(self.world_position)
    }
    // Converts the world position onto the center of the corresponding tile grid cell
    // pub fn to_tile_grid_center(&self) -> IVec2 {
    //     self.to_tile_grid_lb() + IVec2::new(TILESIZE, TILESIZE) / 2
    // }
}
/// Converts the world position onto the left bottom corner of the tile grid
pub fn convert_to_tile_grid(position: Vec2) -> IVec2 {
    IVec2::new(
        (position.x as i32 + TILESIZE / 2).div_euclid(TILESIZE) * TILESIZE,
        (position.y as i32 + TILESIZE / 2).div_euclid(TILESIZE) * TILESIZE,
    )
}
/// Updates the world position of the cursor every frame for other systems to use
fn update_mouse_position(
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    players: Option<Single<&GlobalTransform, (With<Player>, Without<Camera>)>>,
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
    if let Some(player) = players {
        let player_pos = player.translation().xy();
        let mouse_diff = mouse_position.world_position - player_pos;
        let dash_point = if mouse_diff.length_squared() > DASH_RADIUS * DASH_RADIUS {
            player_pos + mouse_diff.normalize() * DASH_RADIUS
        } else {
            mouse_position.world_position
        };
        mouse_position.dash_point = dash_point;
    }
    mouse_position.world_position = position;
}
fn load_level(
    mut events: EventReader<AssetEvent<SaveFile>>,
    mut commands: Commands,
    save_files: Res<Assets<SaveFile>>,
    tiles_q: Query<(Entity, &Sprite, &Transform, &LayerType)>,
    removable: Query<Entity, With<RemoveOnLevelSwap>>,
    textures: Res<map::Textures>,
    maps: Query<(Entity, &TileStorage)>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                for (e, _, _, _) in &tiles_q {
                    commands.entity(e).despawn();
                }
                for e in &removable {
                    commands.entity(e).despawn();
                }
                let level = save_files.get(*id).unwrap();
                for (layer_type, tiles) in &level.layers {
                    let rules = &textures.pack[layer_type].rules;
                    for tile in &tiles.tiles {
                        let rule = rules.iter().find(|rule| rule.target_index == tile.index);
                        let e = commands
                            .spawn(spawn_tile(tile.pos, &textures, tile.index, *layer_type))
                            .id();
                        if let Some(rule) = rule {
                            commands.trigger_targets(*rule, e);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
