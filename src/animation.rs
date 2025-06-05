use std::ops::Range;

use bevy::prelude::*;

use crate::screens::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (execute_animations).run_if(in_state(GameState::Running)),
    );
}
#[derive(Component)]
pub struct AnimationConfig {
    index: Range<usize>,
    frame_timer: Timer,
}
impl AnimationConfig {
    pub fn new(index: Range<usize>, fps: u8) -> Self {
        AnimationConfig {
            index,
            frame_timer: Timer::from_seconds(1. / fps as f32, TimerMode::Once),
        }
    }
}
fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == config.index.end - 1 {
                    atlas.index = config.index.start;
                } else {
                    atlas.index += 1;
                }
                config.frame_timer.reset();
            }
        }
    }
}
