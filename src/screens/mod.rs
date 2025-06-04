use bevy::prelude::*;
mod loading;
pub fn plugin(app: &mut App) {
    app.init_state::<GameState>().add_plugins(loading::plugin);
}
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    AssetLoading,
    Running,
}
