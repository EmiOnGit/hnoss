use bevy::math::IVec2;

use crate::map::TILESIZE;

pub fn iter_grid_rect(start: IVec2, end: IVec2) -> Vec<IVec2> {
    let start = start / TILESIZE;
    let end = end / TILESIZE;
    let mut v = Vec::with_capacity((start - end).abs().element_product() as usize);
    let x_s = start.x.min(end.x);
    let x_e = start.x.max(end.x);
    let y_s = start.y.min(end.y);
    let y_e = start.y.max(end.y);
    for x in x_s..=x_e {
        for y in y_s..=y_e {
            v.push(IVec2::new(x * TILESIZE, y * TILESIZE))
        }
    }
    v
}
