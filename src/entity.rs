use crate::{animation::AnimationConfig, editor::SaveOverride, io, map::convert_to_tile_grid};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(apply_rule);
}
#[derive(Reflect, Clone, Copy, Debug)]
pub enum OnSpawnTrigger {
    Fire,
    Collider,
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
fn apply_rule(trigger: Trigger<Rule>, mut commands: Commands, mut sprite: Query<&mut Transform>) {
    let entity = trigger.target();
    let rule = trigger.event();
    let trigger = rule.on_spawn;
    match trigger {
        OnSpawnTrigger::Fire => {
            let mut transform = sprite.get_mut(entity).unwrap();
            transform.translation.y += 4.;
            let position = convert_to_tile_grid(transform.translation.xy());
            let tile = io::Tile {
                pos: position,
                index: rule.target_index,
            };
            commands
                .entity(entity)
                .insert((fire_spawn(), SaveOverride(tile)))
        }
        OnSpawnTrigger::Collider => todo!(),
    };
    warn!("apply {trigger:?} entity {entity:?}");
}
fn fire_spawn() -> impl Bundle {
    (AnimationConfig::new(0..4, 2),)
}
