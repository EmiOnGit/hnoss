mod animation;
mod asset_loading;
mod combat;
mod editor;
mod entity;
mod io;
mod map;
mod movement;
mod screens;
mod utils;
mod widget;
use avian2d::prelude::RigidBody;
use bevy::asset::AssetMetaCheck;
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
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        screens::plugin,
        combat::plugin,
        movement::plugin,
        animation::plugin,
        asset_loading::plugin,
        map::plugin,
        editor::plugin,
    ))
    .add_systems(Startup, (init_camera, init_gizmo));
}
fn init_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1. / 4.;
    commands.spawn((
        Camera2d,
        Camera {
            // hdr: true,
            ..default()
        },
        Projection::Orthographic(projection.clone()),
        MainCamera,
        RigidBody::Kinematic,
    ));
}
fn init_gizmo(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width += 2.;
}
#[derive(Component)]
struct MainCamera;
