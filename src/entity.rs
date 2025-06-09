use crate::{
    animation::{self, Action, AnimationConfig, EnemyAnimation, PlayerAnimation},
    combat::Tame,
    editor::{RemoveOnLevelSwap, SaveOverride},
    io,
    map::{self, ENEMYSIZE, LayerType, TILESIZE},
    movement::CollisionLayer,
    screens::GameState,
    utils::tile_to_world,
};
use avian2d::prelude::{
    self as avian, CollidingEntities, CollisionEventsEnabled, CollisionLayers, Sensor,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

pub fn plugin(app: &mut App) {
    app.add_observer(apply_rule)
        .init_resource::<TowerCountdown>()
        .add_systems(
            Update,
            (check_enemy_spawn).run_if(in_state(GameState::Running)),
        );
}
#[derive(Reflect, Clone, Copy, Debug)]
pub enum OnSpawnTrigger {
    Player,
    PlayerSpawnPlatform,
    Tower,
    Collider,
    Pit,
    Enemy,
    Portal,
    GameFinishedPlatform,
}
#[derive(Reflect, Event, Debug, Clone, Copy)]
pub struct Rule {
    pub target_index: usize,
    pub spawn_in_tilemap: bool,
    pub on_spawn: OnSpawnTrigger,
}
impl Rule {
    pub fn new(target_index: usize, on_spawn: OnSpawnTrigger, spawn_in_tilemap: bool) -> Rule {
        Rule {
            target_index,
            spawn_in_tilemap,
            on_spawn,
        }
    }
}
#[derive(Component)]
pub struct Pit {
    pub can_dash_over: bool,
}
fn apply_rule(
    trigger: Trigger<Rule>,
    mut commands: Commands,
    transf: Query<&mut Transform>,
    tile_positions: Query<&TilePos>,
    player_spawn_platforms: Query<Entity, With<PlayerSpawnPlatform>>,
    textures: Res<map::Textures>,
    mut tile_map: Query<(Entity, &LayerType), With<TileStorage>>,
    players: Query<(Entity, &ChildOf), With<Player>>,
) {
    let entity = trigger.target();
    let rule = trigger.event();
    let trigger = rule.on_spawn;
    let (entities_tilemap_e, _) = tile_map
        .iter_mut()
        .find(|(_, layer_type)| **layer_type == LayerType::Entities)
        .unwrap();
    let entities_tilemap_translation = transf.get(entities_tilemap_e).unwrap().translation;
    match trigger {
        OnSpawnTrigger::Tower => {
            let tile_pos = tile_positions.get(entity).unwrap();
            let mut tower_position = tile_to_world(tile_pos, entities_tilemap_translation);
            tower_position.y += 4.;
            let sprite = Sprite::from_atlas_image(
                textures.fire.texture.clone(),
                TextureAtlas {
                    layout: textures.fire.layout.clone(),
                    index: 0,
                },
            );
            let tile = io::Tile {
                pos: tile_pos.into(),
                index: rule.target_index,
            };
            commands.spawn((
                RemoveOnLevelSwap,
                sprite,
                tower_spawn(),
                Transform::from_translation(tower_position),
                SaveOverride(tile),
            ));
        }
        OnSpawnTrigger::Collider => {
            let tile_pos = tile_positions.get(entity).unwrap();
            let position = tile_to_world(tile_pos, entities_tilemap_translation);
            commands.entity(entity).insert((
                avian::RigidBody::Static,
                avian::Collider::rectangle(TILESIZE as f32, TILESIZE as f32),
                Transform::from_translation(position),
            ));
        }
        OnSpawnTrigger::Player | OnSpawnTrigger::PlayerSpawnPlatform => {
            for (_player, parent) in &players {
                commands.entity(parent.0).despawn();
            }
            for entity in &player_spawn_platforms {
                commands.entity(entity).despawn();
            }
            let tile_pos = tile_positions.get(entity).unwrap();
            let entity = if matches!(trigger, OnSpawnTrigger::PlayerSpawnPlatform) {
                commands.entity(entity).insert(PlayerSpawnPlatform);
                commands.spawn_empty().id()
            } else {
                entity
            };
            let player_position = tile_to_world(tile_pos, entities_tilemap_translation);
            commands
                .spawn((
                    PlayerController,
                    Visibility::Inherited,
                    RemoveOnLevelSwap,
                    avian::RigidBody::Kinematic,
                    avian::Collider::rectangle(5.0, 5.0),
                    Transform::from_translation(player_position + Vec3::Z),
                ))
                .add_child(entity);
            info!("spawn player");
            let sprite = Sprite::from_atlas_image(
                textures.player.texture.clone(),
                TextureAtlas {
                    layout: textures.player.layout.clone(),
                    index: 0,
                },
            );
            let tile = io::Tile {
                pos: tile_pos.into(),
                index: rule.target_index,
            };
            commands.entity(entity).insert((
                Transform::from_translation(Vec3::Y * 10.),
                RemoveOnLevelSwap,
                sprite,
                player_spawn(),
                SaveOverride(tile),
            ));
        }
        OnSpawnTrigger::Enemy => {
            let tile_pos = tile_positions.get(entity).unwrap();
            let enemy_position = tile_to_world(tile_pos, entities_tilemap_translation);
            let sprite = Sprite::from_atlas_image(
                textures.enemy.texture.clone(),
                TextureAtlas {
                    layout: textures.enemy.layout.clone(),
                    index: 0,
                },
            );
            let tile = io::Tile {
                pos: tile_pos.into(),
                index: rule.target_index,
            };
            commands.entity(entity).insert((
                RemoveOnLevelSwap,
                Transform::from_translation(enemy_position),
                sprite,
                enemy_spawn(),
                SaveOverride(tile),
            ));
        }
        OnSpawnTrigger::Pit => {
            let tile_pos = tile_positions.get(entity).unwrap();
            let position = tile_to_world(tile_pos, entities_tilemap_translation);
            commands.entity(entity).insert((
                Transform::from_translation(position),
                Pit {
                    can_dash_over: false,
                },
                avian::RigidBody::Static,
                Sensor,
                avian::Collider::rectangle(TILESIZE as f32, TILESIZE as f32),
                CollisionEventsEnabled,
                CollidingEntities::default(),
            ));
        }
        OnSpawnTrigger::Portal => {
            let tile_pos = tile_positions.get(entity).unwrap();
            let position = tile_to_world(tile_pos, entities_tilemap_translation);
            commands.entity(entity).insert((
                Transform::from_translation(position),
                Portal::Closed,
                avian::RigidBody::Static,
                Sensor,
                avian::Collider::rectangle(TILESIZE as f32, TILESIZE as f32),
                CollisionEventsEnabled,
                CollidingEntities::default(),
            ));
        }
        OnSpawnTrigger::GameFinishedPlatform => {
            let tile_pos = tile_positions.get(entity).unwrap();
            let position = tile_to_world(tile_pos, entities_tilemap_translation);
            commands.entity(entity).insert((
                Transform::from_translation(position),
                Portal::Open,
                avian::RigidBody::Static,
                Sensor,
                avian::Collider::rectangle(TILESIZE as f32, TILESIZE as f32),
                CollisionEventsEnabled,
                CollidingEntities::default(),
            ));
        }
    };
}

#[derive(Resource, Default)]
pub struct TowerCountdown {
    pub timer: Option<Timer>,
    pub level_complete: bool,
}
#[derive(Component)]
pub struct GameFinishPortal;
#[derive(Component)]
pub struct Tower {
    // pub active: Option<Timer>,
    pub active: bool,
    pub activatable: bool,
}
impl Default for Tower {
    fn default() -> Self {
        Tower {
            active: false,
            activatable: true,
        }
    }
}
fn tower_spawn() -> impl Bundle {
    (
        Tower::default(),
        Visibility::Hidden,
        AnimationConfig::new(0..4, 2),
    )
}
#[derive(Component, PartialEq, Eq)]
pub enum Portal {
    Closed,
    Open,
}
#[derive(Component)]
pub struct PlayerController;
#[derive(Component)]
pub struct PlayerSpawnPlatform;
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub dash_decrease: f32,
    pub mode: PlayerMode,
}
pub enum PlayerMode {
    Active(Timer),
    Normal,
    Tired(Timer),
}
impl Player {
    pub fn new(speed: f32) -> Player {
        Player {
            speed,
            dash_decrease: 0.01,
            mode: PlayerMode::Normal,
        }
    }
}
fn player_spawn() -> impl Bundle {
    (
        Player::new(3000.),
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
        animation::animation_bundle(EnemyAnimation::Spawn),
        avian::RigidBody::Dynamic,
        avian::LinearVelocity::ZERO,
        CollisionLayers::new(
            CollisionLayer::Enemy,
            [CollisionLayer::Player, CollisionLayer::Block],
        ),
        avian::LockedAxes::ROTATION_LOCKED,
        avian::Collider::rectangle(ENEMYSIZE.x as f32 / 2., 5.),
        Tame,
    )
}
#[derive(Component)]
pub struct Flag;
fn check_enemy_spawn(mut enemies: Query<(&mut EnemyAnimation, &Sprite, &Visibility)>) {
    for (mut animation, sprite, visibility) in &mut enemies {
        if *animation == EnemyAnimation::Spawn
            && *visibility != Visibility::Hidden
            && sprite.texture_atlas.as_ref().unwrap().index
                == animation.as_animation().last_sprite_index()
        {
            *animation = EnemyAnimation::Idle;
        }
    }
}
