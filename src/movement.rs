use bevy::prelude::*;

use crate::{
    MainCamera,
    animation::{AnimationConfig, EnemyAnimation, PlayerAnimation},
    entity::{Enemy, Player},
    screens::GameState,
};
pub fn plugin(app: &mut App) {
    app.init_resource::<CameraController>().add_systems(
        Update,
        (movement, enemy_movement).run_if(in_state(GameState::Running)),
    );
}
fn movement(
    controller: Res<CameraController>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cam: Single<&mut Transform, With<MainCamera>>,
    mut players: Query<
        (&mut Transform, &mut AnimationConfig, &mut PlayerAnimation),
        (With<Player>, Without<MainCamera>),
    >,
) {
    let mut dir = Vec2::ZERO;
    const SPEED: f32 = 2.;
    if keys.pressed(KeyCode::KeyA) {
        dir -= Vec2::X * SPEED;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += Vec2::X * SPEED;
    }
    if keys.pressed(KeyCode::KeyW) {
        dir += Vec2::Y * SPEED;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= Vec2::Y * SPEED;
    }
    if dir == Vec2::ZERO {
        for (_, _, mut animation) in &mut players {
            if animation.eq(&PlayerAnimation::Running) {
                *animation = PlayerAnimation::Idle;
            }
        }
        return;
    }

    match *controller {
        CameraController::Camera => {
            cam.translation += dir.extend(0.);
        }
        CameraController::Player => {
            for (mut player, mut animation_config, mut animation) in &mut players {
                if animation.eq(&PlayerAnimation::Idle) {
                    *animation = PlayerAnimation::Running;
                }
                animation_config.flip_sprites = dir.x < 0.;
                let dif = player.translation.xy() - cam.translation.xy();
                if !MOVEMENT_RECT.contains(dif + dir) {
                    cam.translation += dir.extend(0.);
                }
                player.translation += dir.extend(0.);
            }
        }
    }
}
pub fn enemy_movement(
    mut enemies: Query<
        (&mut Transform, &mut AnimationConfig, &mut EnemyAnimation),
        (With<Enemy>, Without<Player>),
    >,
    players: Query<&Transform, With<Player>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    for (mut transform, mut animation_config, mut enemy_animation) in &mut enemies {
        let delta = player.translation - transform.translation;
        if delta.length() > 100. {
            if !enemy_animation.eq(&EnemyAnimation::Idle) {
                *enemy_animation = EnemyAnimation::Idle;
            }
            continue;
        }
        if delta.length() < 5. {
            if !enemy_animation.eq(&EnemyAnimation::Explode) {
                *enemy_animation = EnemyAnimation::Explode;
            }
            continue;
        }
        if !enemy_animation.eq(&EnemyAnimation::Running) {
            *enemy_animation = EnemyAnimation::Running;
        }
        animation_config.flip_sprites = delta.x < 0.;

        transform.translation += delta.normalize() * 0.2;
    }
}
const MOVEMENT_RECT: Rect = Rect {
    min: Vec2::new(-60., -70.),
    max: Vec2::new(60., 70.),
};
#[derive(Default, Debug, Resource)]
pub enum CameraController {
    #[default]
    Camera,
    Player,
}
impl CameraController {
    pub fn toggle(&mut self) {
        match self {
            CameraController::Camera => *self = CameraController::Player,
            CameraController::Player => *self = CameraController::Camera,
        }
    }
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
