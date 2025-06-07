use crate::{
    editor::EditorMeta,
    entity::{Enemy, Player, PlayerMode, Tower},
    io::SaveFile,
    map::Textures,
    movement::TIRED_TIME,
    screens::GameState,
};
use bevy::prelude::*;
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (despawn_enemies, check_tower, update_player_mode).run_if(in_state(GameState::Running)),
    );
}

#[derive(Component)]
pub struct Tame;

fn update_player_mode(
    mut players: Query<(&mut Player, &mut Sprite)>,
    time: Res<Time>,
    textures: Res<Textures>,
) {
    let delta = time.delta();
    for (mut player, mut sprite) in &mut players {
        match &mut player.mode {
            PlayerMode::Active(timer) => {
                timer.tick(delta);
                if timer.finished() {
                    sprite.image = textures.player.texture.clone();
                    player.mode = PlayerMode::Tired(Timer::new(TIRED_TIME, TimerMode::Once))
                }
            }
            PlayerMode::Normal => {}
            PlayerMode::Tired(timer) => {
                timer.tick(delta);
                if timer.finished() {
                    player.mode = PlayerMode::Normal
                }
            }
        }
    }
}
fn despawn_enemies(
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
        if editor_meta.current_level_index > 100 {
            println!("WON");
            return;
        }

        editor_meta.current_level_index += 1;
        let number = editor_meta.current_level_index.to_string();

        let handle =
            asset_server.load::<SaveFile>(String::from("level/") + "level" + &number + ".ron");
        editor_meta.current_level = handle;
    }
}

#[derive(Component, Debug)]
#[relationship_target(relationship = DashTargeting)]
pub struct DashTargetedBy(Entity);

#[derive(Component, Debug)]
#[relationship(relationship_target = DashTargetedBy)]
pub struct DashTargeting(pub Entity);
