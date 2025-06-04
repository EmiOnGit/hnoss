mod asset_loading;
mod editor;
mod io;
mod map;
mod screens;
mod utils;
mod widget;
use bevy::prelude::*;
use screens::GameState;
fn main() {
    App::new().add_plugins(app_plugin).run();
}

fn app_plugin(app: &mut App) {
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Hnoss".to_string(),
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        screens::plugin,
        asset_loading::plugin,
        map::plugin,
        editor::plugin,
    ))
    .add_systems(Startup, init_camera);
}
fn init_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1. / 4.;
    commands.spawn((Camera2d, Projection::Orthographic(projection), MainCamera));
}
#[derive(Component)]
struct MainCamera;
