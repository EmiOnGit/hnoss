use std::ops::Range;

use bevy::prelude::*;

use crate::screens::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            execute_animations,
            update_animation_graph::<PlayerAnimation>,
            update_animation_graph::<EnemyAnimation>,
        )
            .run_if(in_state(GameState::Running)),
    );
}
pub trait Action: Component {
    fn as_animation(&self) -> AnimationConfig;
}
pub fn animation_bundle(action: impl Action) -> impl Bundle {
    (action.as_animation(), action)
}
fn update_animation_graph<A>(mut actions: Query<(&mut AnimationConfig, &A), Changed<A>>)
where
    A: Action,
{
    for (mut config, action) in &mut actions {
        let flip = config.flip_sprites;
        *config = action.as_animation();
        config.flip_sprites = flip;
    }
}
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub enum PlayerAnimation {
    Idle,
    Running,
}
impl Action for PlayerAnimation {
    fn as_animation(&self) -> AnimationConfig {
        match self {
            PlayerAnimation::Idle => AnimationConfig::new(0..4, 2),
            PlayerAnimation::Running => AnimationConfig::new(6..17, 8),
        }
    }
}
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub enum EnemyAnimation {
    Idle,
    Running,
    Explode,
}
impl Action for EnemyAnimation {
    fn as_animation(&self) -> AnimationConfig {
        match self {
            EnemyAnimation::Idle => AnimationConfig::new(0..2, 2),
            EnemyAnimation::Running => AnimationConfig::new(2..7, 8),
            EnemyAnimation::Explode => AnimationConfig::new(7..13, 5),
        }
    }
}
#[derive(Component, Clone, Debug)]
pub struct AnimationConfig {
    index: Range<usize>,
    frame_timer: Timer,
    pub flip_sprites: bool,
    new: bool,
}
impl AnimationConfig {
    pub fn new(index: Range<usize>, fps: u8) -> Self {
        AnimationConfig {
            index,
            frame_timer: Timer::from_seconds(1. / fps as f32, TimerMode::Once),
            flip_sprites: false,
            new: true,
        }
    }
    pub fn is_new(&mut self) -> bool {
        if self.new {
            self.new = false;
            true
        } else {
            false
        }
    }
}
fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        if config.is_new() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = config.index.start;
            }
        }
        if sprite.flip_x != config.flip_sprites {
            sprite.flip_x = config.flip_sprites;
        }
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index >= config.index.end - 1 {
                    atlas.index = config.index.start;
                } else {
                    atlas.index += 1;
                }
                config.frame_timer.reset();
            }
        }
    }
}
