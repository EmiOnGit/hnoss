use bevy::prelude::*;
mod credits;

mod loading;
mod main;
pub fn plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_plugins((loading::plugin, main::plugin, credits::plugin));
}
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    AssetLoading,
    MainMenu,
    Running,
    Credits,
}
