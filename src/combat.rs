use crate::{
    editor::EditorMeta,
    entity::{Enemy, Tower},
    io::SaveFile,
    screens::GameState,
};
use bevy::prelude::*;
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (explode_slimes, check_tower).run_if(in_state(GameState::Running)),
    );
}

#[derive(Component)]
pub struct Tame;

fn explode_slimes(
    mut commands: Commands,
    enemies: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    mut towers: Query<(&mut Tower, &mut Visibility, &Transform)>,
) {
    for (entity, enemy_transform, sprite) in &enemies {
        // 12 is the last sprite of the explode animation of slimes
        // TODO better way
        if sprite.texture_atlas.as_ref().unwrap().index == 12 {
            for (mut tower, mut tower_visibility, tower_transform) in &mut towers {
                if *tower_visibility == Visibility::Hidden
                    && enemy_transform
                        .translation
                        .distance(tower_transform.translation)
                        < 100.
                {
                    *tower_visibility = Visibility::Visible;
                    tower.set_active(3.);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
fn check_tower(
    mut towers: Query<(&mut Tower, &mut Visibility)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut editor_meta: ResMut<EditorMeta>,
) {
    let mut all_lit = !towers.is_empty();

    for (mut tower, mut visibility) in &mut towers {
        if let Some(timer) = &mut tower.active {
            timer.tick(time.delta());
            if timer.finished() {
                *visibility = Visibility::Hidden;
                tower.active = None;
            }
        } else {
            all_lit = false;
        }
    }
    if all_lit {
        let handle = asset_server.load::<SaveFile>(String::from("level/") + "level1" + ".ron");
        editor_meta.current_level = handle;
        println!("WON");
    }
}
