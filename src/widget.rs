use std::borrow::Cow;

use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, CRIMSON, PALE_GOLDENROD},
    prelude::*,
};

use crate::editor::{OverviewButton, TileButton};
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
