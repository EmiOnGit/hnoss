use bevy::{audio::Volume, prelude::*};

use crate::{
    animation::PlayerAnimation, asset_loading::LoadResource, entity::Portal, screens::GameState,
};
const MAIN_TRACK_PATH: &str = "audio/hnoss_main.ogg";
const DASH_TRACK: &str = "audio/dash.ogg";
const SUCCESS_TRACK: &str = "audio/success.ogg";

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Running), setup)
        .init_resource::<SoundStore>()
        .load_resource::<SoundStore>()
        .add_systems(
            Update,
            (check_events, fade_in).run_if(in_state(GameState::Running)),
        );
}
fn check_events(
    mut commands: Commands,
    player: Single<&PlayerAnimation, Changed<PlayerAnimation>>,
    portals: Query<&Portal, Changed<Portal>>,
    sound_store: Res<SoundStore>,
) {
    if portals.iter().any(|portal| *portal == Portal::Open) {
        println!("OPENED THE GATE");
        commands.spawn((
            AudioPlayer::new(sound_store.success.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: Volume::Linear(1.3),
                speed: 1.2,
                ..default()
            },
        ));
    }
    if **player == PlayerAnimation::Dash {
        commands.spawn((
            AudioPlayer::new(sound_store.dash.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: Volume::Linear(1.3),
                speed: 1.5,
                ..default()
            },
        ));
    }
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(MAIN_TRACK_PATH)),
        PlaybackSettings {
            volume: Volume::Linear(0.0),
            speed: 1.2,
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
        MainMusic,
        AudioFadeIn,
    ));
}

#[derive(Component)]
struct AudioFadeIn;

// Fade effect duration
const FADE_TIME: f32 = 15.0;

// Fades in the audio of entities that has the FadeIn component. Removes the FadeIn component once
// full volume is reached.
fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<AudioFadeIn>>,
    time: Res<Time>,
) {
    let target_volume = 0.3;
    for (mut audio, entity) in audio_sink.iter_mut() {
        let current_volume = audio.volume();
        audio.set_volume(
            current_volume + Volume::Linear(time.delta_secs() / FADE_TIME * target_volume),
        );
        if audio.volume().to_linear() >= target_volume {
            audio.set_volume(Volume::Linear(target_volume));
            commands.entity(entity).remove::<AudioFadeIn>();
        }
    }
}

#[derive(Component)]
struct MainMusic;
#[derive(Component)]
struct EventMusic;
#[derive(Resource, Asset, TypePath)]
struct SoundStore {
    pub dash: Handle<AudioSource>,
    pub success: Handle<AudioSource>,
}
impl FromWorld for SoundStore {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        SoundStore {
            dash: asset_server.load(DASH_TRACK),
            success: asset_server.load(SUCCESS_TRACK),
        }
    }
}
