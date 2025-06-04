use bevy::{
    color::palettes::{self, css::CRIMSON, tailwind::BLUE_900},
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
    sprite::Anchor,
};

use crate::{
    GameState,
    io::{self, SaveFile},
    map::{self, LayerType, MousePosition, TILESIZE, Textures, convert_to_tile_grid},
    utils::iter_grid_rect,
    widget,
};
pub fn plugin(app: &mut App) {
    app.add_event::<EditorEvents>()
        .init_resource::<EditorMeta>()
        .add_systems(
            OnEnter(GameState::Running),
            (init_ui_tile_selection, init_ui_overview),
        )
        .add_systems(
            Update,
            (
                check_input,
                process_editor_events,
                tile_button_system,
                overview_button_system,
                update_scroll_position,
                update_selected_tile,
            )
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update,
            (current_tile_ui, draw_selection_indicator).run_if(in_state(GameState::Running)),
        );
}
#[derive(Resource, Default)]
pub struct EditorMeta {
    selected_tile: Option<TextureAtlas>,
    /// Stores a started selection in world space.
    /// You can select a tile region by pressing LMouse and dragging over a region
    current_selection_start: Option<Vec2>,
    layer_type: LayerType,
    current_level: Handle<SaveFile>,
}
fn check_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut event_writer: EventWriter<EditorEvents>,
    mut editor_meta: ResMut<EditorMeta>,
    ui_q: Query<&Interaction>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if ui_q
            .iter()
            .any(|interaction| *interaction != Interaction::None)
        {
            info!("ignore mousepress because of ui interaction")
        } else {
            let position = mouse_position.world_position;
            editor_meta.current_selection_start = Some(position);
        }
    }
    if mouse.just_released(MouseButton::Left) {
        if let Some(start_pos) = editor_meta.current_selection_start {
            if ui_q
                .iter()
                .any(|interaction| *interaction != Interaction::None)
            {
                info!("ignore mouserelease because of ui interaction");
            } else {
                let current_pos = mouse_position.world_position;
                event_writer.write(EditorEvents::SpawnTiles(start_pos, current_pos));
            }
            editor_meta.current_selection_start = None;
        }
    }
}
fn draw_selection_indicator(
    mut my_gizmos: Gizmos<DefaultGizmoConfigGroup>,
    mouse_position: Res<MousePosition>,
    editor_meta: Res<EditorMeta>,
) {
    let Some(start_pos) = editor_meta.current_selection_start else {
        return;
    };
    let cur_pos = mouse_position.to_tile_grid_center().as_vec2();
    let center = (cur_pos + start_pos) / 2.;
    let size = cur_pos - start_pos;
    my_gizmos.rect_2d(
        Isometry2d::new(center, Rot2::IDENTITY),
        size,
        palettes::tailwind::AMBER_600,
    );
}
fn current_tile_ui(
    mut my_gizmos: Gizmos<DefaultGizmoConfigGroup>,
    mouse_position: Res<MousePosition>,
) {
    let position = mouse_position.to_tile_grid_center();
    my_gizmos.rect_2d(
        Isometry2d::new(position.as_vec2(), Rot2::IDENTITY),
        Vec2::new(TILESIZE as f32, TILESIZE as f32),
        palettes::basic::FUCHSIA,
    );
}
#[derive(Event)]
enum EditorEvents {
    SpawnTiles(Vec2, Vec2),
    SaveLevel { name: String },
    LoadLevel { name: String },
}
pub fn spawn_tile(
    tile_grid_position: IVec2,
    textures: &Textures,
    index: usize,
    layer_type: LayerType,
) -> impl Bundle {
    let mut sprite = Sprite::from_atlas_image(
        textures.textures.clone(),
        TextureAtlas {
            layout: textures.layout.clone(),
            index,
        },
    );
    let position = tile_grid_position.as_vec2().extend(layer_type.z());
    sprite.anchor = Anchor::BottomLeft;
    (
        sprite.clone(),
        Transform::from_translation(position),
        layer_type,
    )
}
fn process_editor_events(
    mut commands: Commands,
    mut events: EventReader<EditorEvents>,
    mut editor_meta: ResMut<EditorMeta>,
    asset_server: Res<AssetServer>,
    mut asset_event_writer: EventWriter<AssetEvent<SaveFile>>,
    textures: Res<map::Textures>,
    tiles_q: Query<(Entity, &Sprite, &Transform, &LayerType)>,
) {
    for event in events.read() {
        match event {
            EditorEvents::SpawnTiles(start, end) => {
                let start_pos = convert_to_tile_grid(*start);
                let end_pos = convert_to_tile_grid(*end);
                let v = iter_grid_rect(start_pos, end_pos);
                info!("spawning {} tiles", v.len());
                let layer_type = editor_meta.layer_type;
                // despawn all tiles that already exist in that layer
                let mut count = 0;
                for (e, _, trans, tile_layer_type) in &tiles_q {
                    if layer_type == *tile_layer_type
                        && v.iter()
                            .any(|new_pos| trans.translation.xy() == new_pos.as_vec2())
                    {
                        count += 1;
                        commands.entity(e).despawn();
                    }
                }
                if count > 0 {
                    info!("despawned {} tiles", count);
                }
                // spawn new tiles
                for position in v {
                    commands.spawn(spawn_tile(
                        position,
                        &textures,
                        editor_meta
                            .selected_tile
                            .as_ref()
                            .map(|atlas| atlas.index)
                            .unwrap_or_default(),
                        editor_meta.layer_type,
                    ));
                }
            }
            EditorEvents::SaveLevel { name } => {
                info!("Saving level {name}");
                let mut level = io::SaveFile::default();
                for (_e, sprite, trans, tile_layer_type) in &tiles_q {
                    let position = convert_to_tile_grid(trans.translation.xy());
                    let layer = level.layers.entry(*tile_layer_type).or_insert(io::Layer {
                        tiles: Vec::default(),
                    });
                    let Some(texture_atlas) = &sprite.texture_atlas else {
                        continue;
                    };
                    layer.tiles.push(io::Tile {
                        pos: position,
                        index: texture_atlas.index,
                    })
                }
                io::save(&level, &name);
            }
            EditorEvents::LoadLevel { name } => {
                let handle = asset_server.load::<SaveFile>(String::from("level/") + name + ".ron");
                if asset_server.is_loaded_with_dependencies(handle.id()) {
                    let _ = asset_event_writer.write(AssetEvent::Modified { id: handle.id() });
                }
                info!("start loading new level '{name}'");
                editor_meta.current_level = handle
            }
        }
    }
}
fn init_ui_overview(mut commands: Commands, editor_meta: Res<EditorMeta>) {
    commands.spawn((
        Node {
            width: Val::Percent(40.),
            height: Val::Percent(10.),
            ..default()
        },
        BackgroundColor(BLUE_900.into()),
        children![
            widget::overview_button(OverviewButton::LayerType, editor_meta.layer_type.name()),
            widget::overview_button(OverviewButton::Save, "Save"),
            widget::overview_button(OverviewButton::Load, "Load"),
        ],
    ));
}

fn init_ui_tile_selection(
    mut commands: Commands,
    textures: Res<map::Textures>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    commands
        .spawn(widget::tile_selection_root(Val::Percent(88.)))
        .with_children(|parent| {
            // scrolling node
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Stretch,
                    align_content: AlignContent::End,
                    overflow: Overflow::scroll_y(), // n.b.
                    height: Val::Percent(100.),
                    width: Val::Percent(100.),
                    ..default()
                })
                .with_children(|parent| {
                    let atlas = texture_atlas_layouts.get(textures.layout.id()).unwrap();
                    for i in 0..atlas.len() / 2 {
                        parent.spawn((
                            widget::tile_container(Val::Percent(10.)),
                            children![
                                widget::tile_image(ImageNode::from_atlas_image(
                                    textures.textures.clone(),
                                    TextureAtlas {
                                        layout: textures.layout.clone(),
                                        index: i * 2,
                                    },
                                )),
                                widget::tile_image(ImageNode::from_atlas_image(
                                    textures.textures.clone(),
                                    TextureAtlas {
                                        layout: textures.layout.clone(),
                                        index: i * 2 + 1,
                                    },
                                ))
                            ],
                        ));
                    }
                });
        });
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct TileButton;
#[derive(Component)]
pub enum OverviewButton {
    LayerType,
    Save,
    Load,
}
fn overview_button_system(
    mut layer_node_q: Query<
        (&Interaction, &mut Text, &mut Outline, &OverviewButton),
        (With<Button>, Changed<Interaction>),
    >,
    mut editor_meta: ResMut<EditorMeta>,
    mut event_writer: EventWriter<EditorEvents>,
) {
    for (interaction, mut text, mut outline, overview_button) in &mut layer_node_q {
        match *interaction {
            Interaction::Pressed => match overview_button {
                OverviewButton::LayerType => {
                    editor_meta.layer_type = editor_meta.layer_type.next();
                    **text = editor_meta.layer_type.name().into();
                }
                OverviewButton::Save => {
                    event_writer.write(EditorEvents::SaveLevel {
                        name: "default".into(),
                    });
                }
                OverviewButton::Load => {
                    event_writer.write(EditorEvents::LoadLevel {
                        name: "default".into(),
                    });
                }
            },
            Interaction::Hovered => match overview_button {
                OverviewButton::LayerType => {
                    text.push_str("  ");
                    text.push_str(editor_meta.layer_type.next().name());
                    outline.color = HOVERED_BUTTON;
                }
                OverviewButton::Save => {
                    outline.color = HOVERED_BUTTON;
                }
                OverviewButton::Load => {
                    outline.color = HOVERED_BUTTON;
                }
            },
            Interaction::None => match overview_button {
                OverviewButton::LayerType => {
                    outline.color = NORMAL_BUTTON;
                    **text = editor_meta.layer_type.name().into();
                }
                OverviewButton::Save => {
                    outline.color = NORMAL_BUTTON;
                }
                OverviewButton::Load => {
                    outline.color = NORMAL_BUTTON;
                }
            },
        }
    }
}
fn tile_button_system(
    mut image_node_q: Query<
        (&Interaction, &mut ImageNode, &mut Outline),
        (With<Button>, With<TileButton>),
    >,
    interaction_q: Query<Entity, (Changed<Interaction>, With<Button>)>,
    mut editor_meta: ResMut<EditorMeta>,
) {
    let mut new_pressed = false;
    for entity in &interaction_q {
        let Ok((interaction, image_node, mut outline)) = image_node_q.get_mut(entity) else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                let Some(atlas) = image_node.texture_atlas.clone() else {
                    continue;
                };
                editor_meta.selected_tile = Some(atlas);
                new_pressed = true;
                outline.color = PRESSED_BUTTON;
            }
            Interaction::Hovered => {
                if editor_meta.selected_tile != image_node.texture_atlas {
                    outline.color = HOVERED_BUTTON;
                }
            }
            Interaction::None => {
                if editor_meta.selected_tile != image_node.texture_atlas {
                    outline.color = NORMAL_BUTTON;
                }
            }
        }
    }
    if new_pressed {
        for (_, image_node, mut outline) in &mut image_node_q {
            if editor_meta.selected_tile != image_node.texture_atlas {
                if outline.color == PRESSED_BUTTON {
                    outline.color = NORMAL_BUTTON;
                }
            }
        }
    }
}

/// Updates the selected tile of image nodes in response to mouse input
fn update_selected_tile(
    mouse: Res<ButtonInput<MouseButton>>,
    hover_map: Res<HoverMap>,
    mut editor_meta: ResMut<EditorMeta>,
    ui_images: Query<&ImageNode>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                let Ok(layout) = ui_images.get(*entity) else {
                    continue;
                };
                // the texture atlas is always of variant `Option::Some`
                editor_meta.selected_tile = layout.texture_atlas.clone();
                info!(
                    "changed selected tile to {:?}",
                    layout.texture_atlas.as_ref().unwrap().index
                );
            }
        }
    }
}
/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (
                mouse_wheel_event.x * TILESIZE as f32,
                mouse_wheel_event.y * TILESIZE as f32,
            ),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::math::{ivec2, vec2};

    #[test]
    fn test_convert_to_tile_grid() {
        assert_eq!(super::convert_to_tile_grid(vec2(1., 1.)), ivec2(0, 0));
        assert_eq!(super::convert_to_tile_grid(vec2(1., -1.)), ivec2(0, -16));
        assert_eq!(super::convert_to_tile_grid(vec2(22., -1.)), ivec2(16, -16));
        assert_eq!(
            super::convert_to_tile_grid(vec2(-22., -16.)),
            ivec2(-32, -16)
        );
    }
}
