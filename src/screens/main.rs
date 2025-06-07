use bevy::prelude::*;

use crate::{map::Textures, screens::GameState, widget};
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, textures: Res<Textures>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(GameState::MainMenu),
        #[cfg(not(target_family = "wasm"))]
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ImageNode::new(textures.main_menu_image.clone())
            ),
            widget::button("Play", enter_gameplay_screen),
            widget::button("Credits", open_credits_menu),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ImageNode::new(asset_server.load_with_settings(
                    // This should be an embedded asset for instant loading, but that is
                    // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                    "goblin_splash.jpg",
                    |settings: &mut ImageLoaderSettings| {
                        // Make an exception for the splash image in case
                        // `ImagePlugin::default_nearest()` is used for pixel art.
                        settings.sampler = ImageSampler::linear();
                    },
                ))
            ),
            widget::button("Play", enter_loading_or_gameplay_screen),
            widget::button("Credits", open_credits_menu),
        ],
    ));
}
fn enter_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
) {
    next_screen.set(GameState::Running);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<GameState>>) {
    next_menu.set(GameState::Credits);
}
