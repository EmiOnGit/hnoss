use super::GameState;
use crate::{asset_loading, widget};
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

const LOADING_BACKGROUND_COLOR: Color = Color::srgb(0.857, 0.857, 0.857);
const LOADING_DURATION_SECS: f32 = 1.2;
const LOADING_FADE_DURATION_SECS: f32 = 0.6;

pub fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(LOADING_BACKGROUND_COLOR));
    app.add_systems(OnEnter(GameState::AssetLoading), spawn_loading_screen);

    // fading of loading screen image
    app.add_systems(
        Update,
        (fade_in_and_out).run_if(in_state(GameState::AssetLoading)),
    );
    // state keeping of loading timer
    app.register_type::<LoadingTimer>()
        .add_systems(OnEnter(GameState::AssetLoading), insert_loading_timer)
        .add_systems(OnExit(GameState::AssetLoading), remove_loading_timer)
        .add_systems(
            Update,
            (tick_loading_timer).run_if(in_state(GameState::AssetLoading)),
        );
    // gamestate transition
    app.add_systems(
        Update,
        enter_gameplay_screen.run_if(
            in_state(GameState::AssetLoading)
                .and(asset_loading::all_assets_loaded)
                .and(check_loading_timer),
        ),
    )
    // Exit the splash screen early if the player hits escape.
    .add_systems(
        Update,
        enter_gameplay_screen.run_if(
            in_state(GameState::AssetLoading)
                .and(input_just_pressed(KeyCode::Escape))
                .and(asset_loading::all_assets_loaded),
        ),
    );
}
fn enter_gameplay_screen(mut commands: Commands) {
    info!(
        "finished loading assets
        start gameloop"
    );
    commands.set_state(GameState::Running);
}
fn spawn_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        BackgroundColor(LOADING_BACKGROUND_COLOR),
        StateScoped(GameState::AssetLoading),
        children![(
            Name::new("Loading image"),
            Node {
                margin: UiRect::all(Val::Auto),
                width: Val::Percent(70.0),
                ..default()
            },
            ImageNode::new(asset_server.load_with_settings(
                // This should be an embedded asset for instant loading, but that is
                // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                "splash.jpg",
                |settings: &mut ImageLoaderSettings| {
                    // Make an exception for the splash image in case
                    // `ImagePlugin::default_nearest()` is used for pixel art.
                    settings.sampler = ImageSampler::linear();
                },
            )),
            ImageNodeFadeInOut {
                total_duration: LOADING_DURATION_SECS,
                fade_duration: LOADING_FADE_DURATION_SECS,
                t: 0.0,
            },
        )],
    ));
}
#[derive(Component, Reflect)]
#[reflect(Component)]
struct ImageNodeFadeInOut {
    /// Total duration in seconds.
    total_duration: f32,
    /// Fade duration in seconds.
    fade_duration: f32,
    /// Current progress in seconds, between 0 and [`Self::total_duration`].
    t: f32,
}

impl ImageNodeFadeInOut {
    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

fn fade_in_and_out(
    time: Res<Time>,
    mut animation_query: Query<(&mut ImageNodeFadeInOut, &mut ImageNode)>,
) {
    for (mut anim, mut image) in &mut animation_query {
        anim.t += time.delta_secs();
        image.color.set_alpha(anim.alpha())
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct LoadingTimer(Timer);

impl Default for LoadingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(LOADING_DURATION_SECS, TimerMode::Once))
    }
}

fn insert_loading_timer(mut commands: Commands) {
    commands.init_resource::<LoadingTimer>();
}

fn remove_loading_timer(mut commands: Commands) {
    commands.remove_resource::<LoadingTimer>();
}

fn tick_loading_timer(time: Res<Time>, mut timer: ResMut<LoadingTimer>) {
    timer.0.tick(time.delta());
}

fn check_loading_timer(timer: Res<LoadingTimer>) -> bool {
    timer.0.finished()
}
