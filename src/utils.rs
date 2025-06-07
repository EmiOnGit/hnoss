use bevy::math::{IVec2, Vec2, Vec3};
use bevy_ecs_tilemap::{
    map::{TilemapTileSize, TilemapType},
    tiles::TilePos,
};

use crate::map::{TILEMAP_ANCHOR, TILEMAP_MAPSIZE, TILESIZE};

pub fn iter_grid_rect(start: TilePos, end: TilePos) -> Vec<TilePos> {
    let start = IVec2::new(start.x as i32, start.y as i32);
    let end = IVec2::new(end.x as i32, end.y as i32);
    let mut v = Vec::with_capacity((start - end).abs().element_product() as usize);
    let x_s = start.x.min(end.x);
    let x_e = start.x.max(end.x);
    let y_s = start.y.min(end.y);
    let y_e = start.y.max(end.y);
    for x in x_s..=x_e {
        for y in y_s..=y_e {
            v.push(TilePos::new(x as u32, y as u32))
        }
    }
    v
}
pub fn tile_to_world(tile: &TilePos, tilemap_translation: Vec3) -> Vec3 {
    let tile_size = TilemapTileSize {
        x: TILESIZE as f32,
        y: TILESIZE as f32,
    };
    tile.center_in_world(
        &TILEMAP_MAPSIZE.into(),
        &tile_size.into(),
        &tile_size,
        &TilemapType::Square,
        &TILEMAP_ANCHOR,
    )
    .extend(0.)
        + tilemap_translation
}
pub fn world_to_tilepos(position: Vec2, tilemap_translation: Vec2) -> Option<TilePos> {
    let tile_size = TilemapTileSize {
        x: TILESIZE as f32,
        y: TILESIZE as f32,
    };
    TilePos::from_world_pos(
        &(position - tilemap_translation),
        &TILEMAP_MAPSIZE.into(),
        &tile_size.into(),
        &tile_size,
        &TilemapType::Square,
        &TILEMAP_ANCHOR,
    )
}
