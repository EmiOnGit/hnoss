use crate::{
    animation::{self, AnimationConfig, EnemyAnimation, PlayerAnimation},
    combat::Tame,
    editor::{RemoveOnLevelSwap, SaveOverride},
    io,
    map::{self, ENEMYSIZE, convert_to_tile_grid},
    movement::CollisionLayer,
};
use avian2d::prelude::{self as avian, CollisionLayers};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(apply_rule);
}
#[derive(Reflect, Clone, Copy, Debug)]
pub enum OnSpawnTrigger {
    Player,
    Tower,
    Collider,
    Enemy,
}
#[derive(Reflect, Event, Debug, Clone, Copy)]
pub struct Rule {
    pub target_index: usize,
    pub on_spawn: OnSpawnTrigger,
}
impl Rule {
    pub fn new(target_index: usize, on_spawn: OnSpawnTrigger) -> Rule {
        Rule {
            target_index,
            on_spawn,
        }
    }
}
fn apply_rule(
    trigger: Trigger<Rule>,
    mut commands: Commands,
    mut transf: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    textures: Res<map::Textures>,
) {
    let entity = trigger.target();
    let rule = trigger.event();
    let trigger = rule.on_spawn;
    match trigger {
        OnSpawnTrigger::Tower => {
            let mut transform = transf.get_mut(entity).unwrap();
            transform.translation.y += 4.;
            let mut sprite = sprites.get_mut(entity).unwrap();
            sprite.image = textures.fire.texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: textures.fire.layout.clone(),
                index: 0,
            });
            let position = convert_to_tile_grid(transform.translation.xy());
            let tile = io::Tile {
                pos: position,
                index: rule.target_index,
            };
            commands
                .entity(entity)
                .insert((tower_spawn(), SaveOverride(tile)));
        }
        OnSpawnTrigger::Collider => todo!(),
        OnSpawnTrigger::Player => {
            let mut transform = transf.get_mut(entity).unwrap();
            commands
                .spawn((
                    RemoveOnLevelSwap,
                    avian::RigidBody::Kinematic,
                    avian::Collider::rectangle(5.0, 5.0),
                    *transform,
                ))
                .add_child(entity);
            *transform = Transform::from_translation(Vec3::new(0., 10., 0.));
            info!("spawn player");
            let mut sprite = sprites.get_mut(entity).unwrap();
            sprite.image = textures.player.texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: textures.player.layout.clone(),
                index: 0,
            });
            let position = convert_to_tile_grid(transform.translation.xy());
            let tile = io::Tile {
                pos: position,
                index: rule.target_index,
            };
            commands
                .entity(entity)
                .insert((player_spawn(), SaveOverride(tile)));
        }
        OnSpawnTrigger::Enemy => {
            let transform = transf.get(entity).unwrap();
            let mut sprite = sprites.get_mut(entity).unwrap();
            sprite.image = textures.enemy.texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: textures.enemy.layout.clone(),
                index: 0,
            });
            let position = convert_to_tile_grid(transform.translation.xy());
            let tile = io::Tile {
                pos: position,
                index: rule.target_index,
            };
            commands
                .entity(entity)
                .insert((enemy_spawn(), SaveOverride(tile)));
        }
    };
    warn!("apply {trigger:?} entity {entity:?}");
}
#[derive(Component, Default)]
pub struct Tower {
    pub active: Option<Timer>,
}
impl Tower {
    pub fn set_active(&mut self, active_time_sec: f32) {
        self.active = Some(Timer::from_seconds(active_time_sec, TimerMode::Once));
    }
}
fn tower_spawn() -> impl Bundle {
    (
        Tower::default(),
        Visibility::Hidden,
        AnimationConfig::new(0..4, 2),
    )
}
#[derive(Component)]
pub struct Player {
    pub speed: f32,
}
impl Player {
    pub fn new(speed: f32) -> Player {
        Player { speed }
    }
}
fn player_spawn() -> impl Bundle {
    (
        Player::new(70.),
        CollisionLayers::new(CollisionLayer::Player, CollisionLayer::Block),
        animation::animation_bundle(PlayerAnimation::Idle),
    )
}
#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
}
fn enemy_spawn() -> impl Bundle {
    (
        Enemy { speed: 3000. },
        animation::animation_bundle(EnemyAnimation::Idle),
        avian::RigidBody::Dynamic,
        avian::LinearVelocity::ZERO,
        CollisionLayers::new(
            CollisionLayer::Enemy,
            [CollisionLayer::Enemy, CollisionLayer::Block],
        ),
        avian::LockedAxes::ROTATION_LOCKED,
        avian::Collider::rectangle(ENEMYSIZE.x as f32 / 2., 5.),
        Tame,
    )
}
#[derive(Component)]
pub struct Flag;
