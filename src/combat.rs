use crate::{
    editor::{EditorMeta, RemoveOnLevelSwap},
    entity::{Enemy, Player, PlayerMode, Tower},
    io::SaveFile,
    map::Textures,
    movement::TIRED_TIME,
    screens::GameState,
};
use bevy::{color::palettes::tailwind::PURPLE_50, prelude::*};
pub const ENEMY_EXPLOSION_RADIUS: f32 = 55.;
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            despawn_enemies,
            check_tower,
            update_player_mode,
            update_explosion_indicator,
        )
            .run_if(in_state(GameState::Running)),
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
            commands.spawn((
                ExplosionIndicator {
                    timer: Timer::from_seconds(1., TimerMode::Once),
                    position: enemy_transform.translation.xy(),
                },
                RemoveOnLevelSwap,
            ));
            for (mut tower, mut tower_visibility, tower_transform) in &mut towers {
                if *tower_visibility == Visibility::Hidden
                    && tower.activatable
                    && enemy_transform
                        .translation
                        .distance(tower_transform.translation)
                        < ENEMY_EXPLOSION_RADIUS
                {
                    *tower_visibility = Visibility::Visible;
                    tower.set_active(3.);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct ExplosionIndicator {
    timer: Timer,
    position: Vec2,
}
fn update_explosion_indicator(
    mut commands: Commands,
    mut explosion_indicator: Query<(Entity, &mut ExplosionIndicator)>,
    time: Res<Time>,
    mut my_gizmos: Gizmos<DefaultGizmoConfigGroup>,
) {
    let delta = time.delta();
    for (e, mut indicator) in &mut explosion_indicator {
        indicator.timer.tick(delta);
        if indicator.timer.finished() {
            commands.entity(e).despawn();
        } else {
            my_gizmos.circle_2d(
                Isometry2d::from_translation(indicator.position),
                ENEMY_EXPLOSION_RADIUS - 5.,
                PURPLE_50.with_alpha(0.1),
            );
        }
    }
}
fn check_tower(
    mut towers: Query<(&mut Tower, &mut Visibility)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut editor_meta: ResMut<EditorMeta>,
) {
    let mut all_lit = !towers.is_empty() && towers.iter().any(|(tower, _)| tower.activatable);

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
        // we need to dirty flag them to avoid a possible level skip
        for (mut tower, _) in &mut towers {
            tower.activatable = false;
        }
        let count = towers.iter().count();
        println!(
            "all lit on level {} with {} towers",
            editor_meta.current_level_index, count
        );
        if editor_meta.current_level_index > 20 {
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
