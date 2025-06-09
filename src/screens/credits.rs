//! The credits menu.

use bevy::{
    ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*,
};

use crate::{screens::GameState, widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(GameState::Credits).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Credits GameState"),
        GlobalZIndex(2),
        StateScoped(GameState::Credits),
        children![
            widget::header("Created by"),
            created_by(),
            widget::header("Assets"),
            assets(),
            widget::button("Back", go_back_on_click),
        ],
    ));
}

fn created_by() -> impl Bundle {
    grid(vec![["Emanuel Boehm", ""]])
}

fn assets() -> impl Bundle {
    grid(vec![
        ["sprites", "CC0 by Emanuel Boehm"],
        ["images", "CC0 by Emanuel Boehm"],
        ["music", "CC0 by Emanuel Boehm"],
    ])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            |(i, text)| {
                (
                    widget::label(text),
                    Node {
                        justify_self: if i % 2 == 0 {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn go_back_on_click(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<GameState>>) {
    next_menu.set(GameState::MainMenu);
}

fn go_back(mut next_menu: ResMut<NextState<GameState>>) {
    next_menu.set(GameState::MainMenu);
}
