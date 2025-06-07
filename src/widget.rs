use std::borrow::Cow;

use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, CRIMSON, PALE_GOLDENROD},
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};

use crate::editor::{NORMAL_BUTTON, OverviewButton, TileButton};
/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}
/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        (
            Node {
                width: Val::Px(220.0),
                height: Val::Px(50.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::MAX,
        ),
    )
}
/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(NORMAL_BUTTON),
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}
pub fn tile_selection_root(left: Val) -> impl Bundle {
    (
        Node {
            left,

            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,

            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,

            ..default()
        },
        BackgroundColor(PALE_GOLDENROD.with_alpha(0.2).into()),
    )
}
pub fn tile_container(height: Val) -> impl Bundle {
    (
        Node {
            min_width: height,
            max_width: height,
            width: height,
            // max_height: height,
            // min_height: height,
            ..default()
        },
        Pickable {
            should_block_lower: false,
            ..default()
        },
    )
}
pub fn tile_image(image_node: ImageNode) -> impl Bundle {
    (
        Button,
        TileButton,
        image_node,
        BackgroundColor(ANTIQUE_WHITE.into()),
        Outline::new(Val::Px(4.0), Val::ZERO, NORMAL_BUTTON),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            margin: UiRect::all(Val::Px(4.)),
            ..default()
        },
    )
}
pub fn overview_button(overview_button: OverviewButton, text: impl Into<String>) -> impl Bundle {
    (
        Button,
        Text::new(text),
        TextLayout::new_with_justify(JustifyText::Center),
        overview_button,
        Outline::new(Val::Px(4.0), Val::ZERO, CRIMSON.into()),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            margin: UiRect::all(Val::Px(4.)),
            ..default()
        },
    )
}
pub fn header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
    )
}
/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(24.0),
    )
}
