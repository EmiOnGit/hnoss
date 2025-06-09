use std::{f32::consts::PI, time::Duration};

use avian2d::{
    math::{AdjustPrecision, Scalar, Vector},
    prelude::{
        ColliderOf, CollidingEntities, Collisions, Gravity, LinearVelocity, NarrowPhaseSet,
        PhysicsLayer, PhysicsSchedule, Position, RigidBody, Sensor,
    },
};
use bevy::{
    color::palettes::tailwind::{PURPLE_400, RED_400, YELLOW_400},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

use crate::{
    MainCamera,
    animation::{AnimationConfig, EnemyAnimation, PlayerAnimation},
    combat::{DashTargetedBy, DashTargeting, Tame},
    editor::{EditorEvents, EditorMeta},
    entity::{Enemy, Pit, Player, PlayerController, PlayerMode, Portal},
    io::SaveFile,
    map::{MousePosition, Textures},
    screens::GameState,
};
pub const DASH_RADIUS: f32 = 70.;
pub const DASH_RECOGNITION_RADIUS: f32 = 50.;
pub const DASH_IMPULSE: f32 = 350.;
pub const TIRED_TIME: Duration = Duration::from_secs(2);
pub const ACTIVE_TIME: Duration = Duration::from_secs(3);
pub fn plugin(app: &mut App) {
    app.add_plugins(avian2d::PhysicsPlugins::default().with_length_unit(1.))
        // .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vector::ZERO))
        .add_systems(
            Update,
            (
                dash.run_if(
                    input_just_pressed(KeyCode::Space)
                        .and(|editor_meta: Res<EditorMeta>| editor_meta.edit_mode)
                        .and(in_state(GameState::Running)),
                ),
                dash.run_if(
                    input_just_pressed(MouseButton::Left)
                        .and(|editor_meta: Res<EditorMeta>| !editor_meta.edit_mode)
                        .and(in_state(GameState::Running)),
                ),
            ),
        )
        .add_systems(
            Update,
            (
                movement,
                move_camera,
                check_dash,
                enemy_movement,
                dash_ui,
                check_collisions,
            )
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(
            PhysicsSchedule,
            kinematic_controller_collisions.in_set(NarrowPhaseSet::Last),
        );
}
#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Block,
    Enemy,
    Player,
}
fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut velocities: Query<&mut LinearVelocity>,
    mut players: Query<
        (
            &mut AnimationConfig,
            &mut PlayerAnimation,
            &Player,
            &ChildOf,
        ),
        Without<MainCamera>,
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
        for (_, mut animation, _, parent) in &mut players {
            if animation.eq(&PlayerAnimation::Running) {
                let mut velocity = velocities.get_mut(parent.0).unwrap();
                velocity.0 = Vec2::ZERO;
                *animation = PlayerAnimation::Idle;
            }
        }
        return;
    }

    let delta = time.delta_secs();
    for (mut animation_config, mut animation, player, parent) in &mut players {
        if animation.eq(&PlayerAnimation::Idle) {
            *animation = PlayerAnimation::Running;
        }
        if animation.eq(&PlayerAnimation::Dash) {
            continue;
        }
        if animation.eq(&PlayerAnimation::DashSprint) {
            continue;
        }
        animation_config.flip_sprites = dir.x < 0.;
        let mut velocity = velocities.get_mut(parent.0).unwrap();
        velocity.0 = dir * player.speed * delta;
    }
}
pub fn move_camera(
    cam: Single<(&mut LinearVelocity, &Transform), With<MainCamera>>,
    player: Option<Single<(&GlobalTransform, &Player)>>,
    time: Res<Time>,
) {
    let (mut cam_velocity, cam_transform) = cam.into_inner();
    let Some(player) = player else {
        cam_velocity.0 = Vec2::ZERO;
        return;
    };
    let player_transform = player.0;
    let delta = time.delta_secs();

    let diff = player_transform.translation().xy() - cam_transform.translation.xy();
    if !MOVEMENT_RECT.contains(diff) {
        const CAM_RIGIDNESS: f32 = 80.;
        cam_velocity.0 = diff * player.1.speed * delta / CAM_RIGIDNESS;
        let outer_rect = MOVEMENT_RECT.inflate(1.3);
        if !outer_rect.contains(diff) {
            cam_velocity.0 = diff * player.1.speed * delta / CAM_RIGIDNESS * 2.;
        }
        let outer_rect = MOVEMENT_RECT.inflate(1.6);
        if !outer_rect.contains(diff) {
            cam_velocity.0 = diff * player.1.speed * delta / CAM_RIGIDNESS * 4.;
        }
    } else {
        cam_velocity.0 = Vec2::ZERO;
    }
}
pub fn enemy_movement(
    mut enemies: Query<
        (
            &Transform,
            &mut LinearVelocity,
            &mut AnimationConfig,
            &mut EnemyAnimation,
            &Enemy,
        ),
        (Without<Player>, Without<Tame>),
    >,
    players: Query<&GlobalTransform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(player) = players.single() else {
        return;
    };

    let delta_time = time.delta_secs();
    for (transform, mut linear_velocity, mut animation_config, mut enemy_animation, enemy) in
        &mut enemies
    {
        if enemy_animation.eq(&EnemyAnimation::Explode)
            || enemy_animation.eq(&EnemyAnimation::DashTargeted)
        {
            continue;
        }
        let delta = player.translation().xy() - transform.translation.xy();
        if delta.length() > 100. {
            linear_velocity.0 = Vec2::ZERO;
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

        let normalized = delta.normalize();
        linear_velocity.0 = (linear_velocity.0 + normalized * delta_time * enemy.speed) / 2.;
    }
}
const MOVEMENT_RECT: Rect = Rect {
    min: Vec2::new(-30., -30.),
    max: Vec2::new(30., 30.),
};

fn dash(
    mut commands: Commands,
    mouse_position: Res<MousePosition>,
    players: Single<(&mut LinearVelocity, &GlobalTransform, &Children)>,
    mut player_comp: Query<(Entity, &mut PlayerAnimation, &Player)>,
    mut enemies: Query<
        (Entity, &GlobalTransform, &mut EnemyAnimation, &Visibility),
        (With<Enemy>, Without<Player>),
    >,
) {
    let enemy_pos = |trans: &GlobalTransform| trans.translation().xy();
    let (mut velocity, global_transform, children) = players.into_inner();
    let Some(child) = children.first() else {
        return;
    };
    let Ok((entity, mut animation, player)) = player_comp.get_mut(*child) else {
        return;
    };
    if !animation.eq(&PlayerAnimation::Dash) && !matches!(player.mode, PlayerMode::Tired(_)) {
        let dash_point = mouse_position.dash_point;
        let player_pos = global_transform.translation().xy();
        let closest_enemy = enemies
            .iter_mut()
            .filter(|(_, enemy, animation, visible)| {
                *visible != Visibility::Hidden
                    && enemy_pos(enemy).distance(player_pos) <= DASH_RADIUS
                    && !animation.eq(&EnemyAnimation::Explode)
            })
            .min_by(|(_, enemy, _, _), (_, enemy2, _, _)| {
                let d1 = enemy_pos(enemy).distance_squared(dash_point);
                let d2 = enemy_pos(enemy2).distance_squared(dash_point);
                d1.total_cmp(&d2)
            });
        let Some((enemy_e, closest_transform, mut closest_animation, _)) = closest_enemy else {
            return;
        };
        if enemy_pos(closest_transform).distance_squared(dash_point)
            > DASH_RECOGNITION_RADIUS * DASH_RECOGNITION_RADIUS
        {
            return;
        }
        let distance = enemy_pos(closest_transform) - player_pos;
        commands.entity(entity).insert(DashTargeting(enemy_e));
        *animation = PlayerAnimation::Dash;
        *closest_animation = EnemyAnimation::DashTargeted;
        velocity.0 = distance.normalize_or_zero() * DASH_IMPULSE;
    }
}

fn check_dash(
    mut commands: Commands,
    mut players: Query<(Entity, &mut LinearVelocity)>,
    mut player_comp: Query<(
        Entity,
        &mut PlayerAnimation,
        &mut Sprite,
        &mut Player,
        &ChildOf,
        Option<&DashTargeting>,
    )>,
    textures: Res<Textures>,
    transforms: Query<&Transform>,
    mut enemy: Option<Single<&mut EnemyAnimation, With<DashTargetedBy>>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (child_player_e, mut animation, mut sprite, mut player, parent, dash_target) in
        &mut player_comp
    {
        if animation.eq(&PlayerAnimation::Dash) {
            let (e, _velo) = players.get_mut(parent.0).unwrap();
            let Some(DashTargeting(target_e)) = dash_target else {
                continue;
            };
            let player_tr = transforms.get(e).unwrap();
            let enemy_tr = transforms.get(*target_e).unwrap();
            if player_tr.translation.distance(enemy_tr.translation) < 20. {
                sprite.image = textures.player_active.texture.clone();
                player.mode = PlayerMode::Active(Timer::new(ACTIVE_TIME, TimerMode::Once));
                commands.entity(child_player_e).remove::<DashTargeting>();
                *animation = PlayerAnimation::DashSprint;
                *enemy.as_mut().unwrap().as_mut() = EnemyAnimation::Explode;
            }
        } else if animation.eq(&PlayerAnimation::DashSprint) {
            let (_e, mut velo) = players.get_mut(parent.0).unwrap();
            velo.0 *= player.dash_decrease.powf(delta);
            if velo.0.length_squared() < player.speed * player.speed * delta * delta {
                *animation = PlayerAnimation::Running;
            }
        }
    }
}
fn dash_ui(
    mut my_gizmos: Gizmos<DefaultGizmoConfigGroup>,
    mouse_position: Res<MousePosition>,
    player: Single<(&Player, &GlobalTransform, &PlayerAnimation)>,
) {
    let (player, transform, animation) = player.into_inner();
    let player_pos = transform.translation().xy();
    let dash_point = mouse_position.dash_point;
    let delta = player_pos - dash_point;
    let color = match animation {
        PlayerAnimation::Dash => RED_400,
        _ => match &player.mode {
            PlayerMode::Active(_timer) => PURPLE_400,
            PlayerMode::Normal => YELLOW_400,
            PlayerMode::Tired(timer) => RED_400.with_alpha(timer.fraction_remaining() / 2.),
        },
    };
    my_gizmos.arc_2d(
        Isometry2d::new(player_pos, Rot2::radians(delta.to_angle() + PI / 2. - 0.1)),
        0.2,
        delta.length(),
        color,
    );
}

fn check_collisions(
    player: Single<(&ChildOf, &PlayerAnimation), With<Player>>,
    pits: Query<(&CollidingEntities, &Pit), Changed<CollidingEntities>>,
    portals: Query<(&CollidingEntities, &Portal), (Without<Pit>, Changed<CollidingEntities>)>,
    mut editor_meta: ResMut<EditorMeta>,
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<EditorEvents>,
) {
    let (player, animation) = *player;
    for (colliding_entities, pit) in &pits {
        if pit.can_dash_over
            && (*animation == PlayerAnimation::Dash || *animation == PlayerAnimation::DashSprint)
        {
            continue;
        }
        if colliding_entities.contains(&player.0) {
            event_writer.write(EditorEvents::RespawnPlayer);
        }
    }
    for (colliding_entities, portal) in &portals {
        if colliding_entities.contains(&player.0) && *portal == Portal::Open {
            editor_meta.current_level_index += 1;
            let number = editor_meta.current_level_index.to_string();

            info!("start loading level {number}");
            let handle =
                asset_server.load::<SaveFile>(String::from("level/") + "level" + &number + ".ron");
            editor_meta.current_level = handle;
        }
    }
}
// DO NOT USE
// I do not know how this works one bit and copied it from the avian examples
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut character_controllers: Query<
        (&mut Position, &mut LinearVelocity),
        (With<RigidBody>, With<PlayerController>),
    >,
    time: Res<Time>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;

        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                character_rb = *bodies.get(rb1).unwrap();
                is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
                character
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                character_rb = *bodies.get(rb2).unwrap();
                is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers.
        if !character_rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;

            // Solve each penetrating contact in the manifold.
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            // For now, this system only handles velocity corrections for collisions against static geometry.
            if is_other_dynamic {
                continue;
            }

            // Determine if the slope is climbable or if it's too steep to walk on.
            let climbable = false;

            if deepest_penetration > 0.0 {
                // If the slope is climbable, snap the velocity so that the character
                // up and down the surface smoothly.
                {
                    // The character is intersecting an unclimbable object, like a wall.
                    // We want the character to slide along the surface, similarly to
                    // a collide-and-slide algorithm.

                    // Don't apply an impulse if the character is moving away from the surface.
                    if linear_velocity.dot(normal) > 0.0 {
                        continue;
                    }

                    // Slide along the surface, rejecting the velocity along the contact normal.
                    let impulse = linear_velocity.reject_from_normalized(normal);
                    linear_velocity.0 = impulse;
                }
            } else {
                // The character is not yet intersecting the other object,
                // but the narrow phase detected a speculative collision.
                //
                // We need to push back the part of the velocity
                // that would cause penetration within the next frame.

                let normal_speed = linear_velocity.dot(normal);

                // Don't apply an impulse if the character is moving away from the surface.
                if normal_speed > 0.0 {
                    continue;
                }

                // Compute the impulse to apply.
                let impulse_magnitude =
                    normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                let mut impulse = impulse_magnitude * normal * 1.2;

                // Apply the impulse differently depending on the slope angle.
                if climbable {
                    // Avoid sliding down slopes.
                    linear_velocity.y -= impulse.y.min(0.0);
                } else {
                    // Avoid climbing up walls.
                    impulse.y = impulse.y.max(0.0);
                    linear_velocity.0 -= impulse;
                }
            }
        }
    }
}
