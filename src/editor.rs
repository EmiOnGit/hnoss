use avian2d::prelude::LinearVelocity;
use bevy::{
    asset::LoadState,
    color::palettes::{self},
    input::{
        common_conditions::input_just_released,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    picking::hover::HoverMap,
    prelude::*,
};
use bevy_ecs_tilemap::{
    map::{TilemapId, TilemapSize},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::{
    GameState, MainCamera,
    animation::{EnemyAnimation, PlayerAnimation},
    combat::{ScreenShake, TRAUMA},
    entity::{Enemy, Player, PlayerMode, TowerCountdown},
    io::{self, SaveFile, Tile},
    map::{
        self, BACKGROUND_COLOR, DEBUG_BACKGROUND_COLOR, LayerType, MousePosition, TILEMAP_OFFSET,
        TILESIZE, convert_to_tile_pos,
    },
    utils::{self, iter_grid_rect, tile_to_world},
    widget::{self, DEAD_BACKGROUND},
};
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
pub fn plugin(app: &mut App) {
    app.add_event::<EditorEvents>()
        .init_resource::<EditorMeta>()
        .add_event::<UiRespawnTrigger>()
        .add_observer(ui_tile_selection_update)
        .add_observer(draw_deads)
        .add_observer(init_ui_overview)
        .add_observer(show_control_image)
        .add_systems(
            Update,
            (|mut commands: Commands| commands.trigger(UiRespawnTrigger::OverviewRespawn))
                .run_if(input_just_released(KeyCode::KeyP).and(in_state(GameState::Running))),
        )
        .add_systems(OnEnter(GameState::Running), |mut commands: Commands| {
            commands.trigger(UiRespawnTrigger::ShowControlImage);
            commands.trigger(UiRespawnTrigger::Deads);
        })
        .add_systems(
            Update,
            (
                debug,
                update_control_images,
                process_editor_events,
                overview_button_system,
                check_level_loaded,
            )
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update,
            (
                current_tile_ui,
                tile_button_system,
                update_selected_tile,
                draw_selection_indicator,
                check_input,
                update_scroll_position,
            )
                .run_if(in_state(GameState::Running).and(|meta: Res<EditorMeta>| meta.edit_mode)),
        );
}
#[derive(Resource, Default)]
pub struct EditorMeta {
    selected_tile: Option<TextureAtlas>,
    /// Stores a started selection in world space.
    /// You can select a tile region by pressing LMouse and dragging over a region
    current_selection_start: Option<Vec2>,
    layer_type: LayerType,
    pub current_level: Handle<SaveFile>,
    pub current_level_index: usize,
    pub edit_mode: bool,
    pub deads: usize,
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
        let Some(start_pos) = editor_meta.current_selection_start else {
            return;
        };
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
#[derive(Component)]
struct PlayerDeadsUiRoot;
fn draw_deads(
    trigger: Trigger<UiRespawnTrigger>,
    mut commands: Commands,
    editor_meta: Res<EditorMeta>,
    mut texts: Query<&mut Text>,
    ui_root: Option<Single<&Children, With<PlayerDeadsUiRoot>>>,
) {
    if !matches!(trigger.event(), UiRespawnTrigger::Deads) {
        return;
    }
    if let Some(ui_root) = &ui_root {
        for child in ui_root.as_ref() {
            let mut text = texts.get_mut(*child).unwrap();
            text.0 = editor_meta.deads.to_string();
        }
    } else {
        commands.spawn((
            Node {
                left: Val::Percent(5.),
                // right: Val::Percent(10.),
                width: Val::Percent(10.),
                height: Val::Percent(10.),
                top: Val::Percent(5.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            PlayerDeadsUiRoot,
            children![widget::player_dead(editor_meta.deads),],
        ));
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
    let cur_pos = utils::world_to_tilepos(mouse_position.world_position, TILEMAP_OFFSET)
        .map(|tilepos| utils::tile_to_world(&tilepos, TILEMAP_OFFSET.extend(0.)).xy())
        .unwrap_or(start_pos);
    let center = (cur_pos + start_pos) / 2.;
    let size = cur_pos - start_pos;
    my_gizmos.rect_2d(
        Isometry2d::new(center, Rot2::IDENTITY),
        size,
        palettes::tailwind::BLUE_100,
    );
}
fn current_tile_ui(
    mut my_gizmos: Gizmos<DefaultGizmoConfigGroup>,
    mouse_position: Res<MousePosition>,
) {
    let position = mouse_position.to_tilepos_vec2();
    if let Some(position) = position {
        my_gizmos.rect_2d(
            Isometry2d::new(position, Rot2::IDENTITY),
            Vec2::new(TILESIZE as f32, TILESIZE as f32),
            palettes::basic::FUCHSIA,
        );
    }
}
#[derive(Event)]
pub enum EditorEvents {
    SpawnTiles(Vec2, Vec2),
    RespawnPlayer,
    SaveLevel,
    /// Name of the file without the extension so 'assets/level1.ron' becomes 'level1'
    /// If [`Option::None`] is provided then a file dialog is opened
    LoadLevel {
        name: Option<String>,
    },
    ToggleEditMode,
}
pub fn spawn_tiled(
    tilemap_entity: Entity,
    position: TilePos,
    index: usize,
    layer_type: LayerType,
) -> impl Bundle {
    (
        TileBundle {
            position,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(index as u32),
            ..default()
        },
        layer_type,
    )
}
/// Overrides the index and position of the tile when storing level to disk
#[derive(Component)]
pub struct SaveOverride(pub io::Tile);
#[derive(Component)]
pub struct RemoveOnLevelSwap;
fn process_editor_events(
    mut commands: Commands,
    mut events: EventReader<EditorEvents>,
    mut editor_meta: ResMut<EditorMeta>,
    asset_server: Res<AssetServer>,
    mut asset_event_writer: EventWriter<AssetEvent<SaveFile>>,
    textures: Res<map::Textures>,
    cam: Single<Entity, With<MainCamera>>,
    tiles_q: Query<(
        Entity,
        &TileTextureIndex,
        &TilePos,
        &LayerType,
        Option<&SaveOverride>,
    )>,
    mut players: Query<(&mut Player, &mut PlayerAnimation, &ChildOf, &SaveOverride)>,
    mut enemies: Query<(&mut Visibility, &mut EnemyAnimation), With<Enemy>>,
    mut parent_player: Query<(&mut LinearVelocity, &mut Transform)>,
    mut tower_timer: ResMut<TowerCountdown>,
    override_tiles: Query<(Entity, &LayerType, &SaveOverride, Option<&ChildOf>)>,
    mut tile_map: Query<(Entity, &mut TileStorage, &TilemapSize, &LayerType)>,
) {
    for event in events.read() {
        match event {
            EditorEvents::SpawnTiles(start, end) => {
                let start_pos = convert_to_tile_pos(*start);
                let end_pos = convert_to_tile_pos(*end);
                let v = iter_grid_rect(start_pos, end_pos);
                info!("spawning {} tiles", v.len());
                let layer_type = editor_meta.layer_type;
                // despawn all tiles that already exist in that layer
                let mut count = 0;
                let rect = URect::new(start_pos.x, start_pos.y, end_pos.x, end_pos.y);
                for (e, _, tile_pos, tile_layer_type, _) in &tiles_q {
                    if layer_type == *tile_layer_type && rect.contains(tile_pos.into()) {
                        count += 1;
                        commands.entity(e).despawn();
                    }
                }
                for (e, tile_layer_type, override_tile, parent) in &override_tiles {
                    if layer_type == *tile_layer_type && rect.contains(override_tile.0.pos) {
                        if let Some(parent) = parent {
                            commands.entity(parent.0).despawn();
                        } else {
                            commands.entity(e).despawn();
                        }
                        count += 1;
                    }
                }
                if count > 0 {
                    info!("despawned {} tiles", count);
                }
                let (tilemap_e, mut storage, tilemap_size, _) = tile_map
                    .iter_mut()
                    .find(|(_e, _storage, _, layer_type)| **layer_type == editor_meta.layer_type)
                    .unwrap();
                // spawn new tiles
                let rules = &textures.pack[&editor_meta.layer_type].rules;
                if let Some(selected_tile) = &editor_meta.selected_tile {
                    let rule_index = rules
                        .iter()
                        .position(|rule| rule.target_index == selected_tile.index)
                        .unwrap_or_default();
                    for tile_pos in v {
                        if tilemap_size.x <= tile_pos.x || tilemap_size.y <= tile_pos.y {
                            continue;
                        }
                        map::spawn_tile(
                            &[rules[rule_index]],
                            &mut commands,
                            &Tile {
                                pos: tile_pos.into(),
                                index: selected_tile.index,
                            },
                            tilemap_e,
                            &mut storage,
                            layer_type,
                        );
                    }
                }
            }
            EditorEvents::SaveLevel => {
                info!("Saving level");
                let mut level = io::SaveFile::default();
                for (_e, texture_index, tile_pos, tile_layer_type, save_override) in &tiles_q {
                    // will processed later
                    if save_override.is_some() {
                        continue;
                    }
                    let layer = level.layers.entry(*tile_layer_type).or_insert(io::Layer {
                        tiles: Vec::default(),
                    });
                    if let Some(tile) = save_override {
                        layer.tiles.push(tile.0);
                        continue;
                    }
                    let index = texture_index.0 as usize;
                    layer.tiles.push(io::Tile {
                        pos: tile_pos.into(),
                        index,
                    })
                }
                for (_e, tile_layer_type, tile, _) in &override_tiles {
                    let layer = level.layers.entry(*tile_layer_type).or_insert(io::Layer {
                        tiles: Vec::default(),
                    });
                    layer.tiles.push(tile.0);
                }
                io::save(&level);
            }
            EditorEvents::LoadLevel { name } => {
                let handle = if let Some(name) = name {
                    info!("start loading new level '{name}'");
                    Some(asset_server.load::<SaveFile>(String::from("level/") + name + ".ron"))
                } else {
                    io::select_file().map(|path| asset_server.load(path))
                };
                if let Some(handle) = handle {
                    // WARN bad practice but we have to fire a assetEvent for the [map::load_level()] to fire again
                    if asset_server.is_loaded_with_dependencies(handle.id()) {
                        let _ = asset_event_writer.write(AssetEvent::Modified { id: handle.id() });
                    }
                    editor_meta.current_level = handle;
                } else {
                    info!("loading failed");
                }
            }
            EditorEvents::ToggleEditMode => {
                commands.trigger(UiRespawnTrigger::OverviewRespawn);
                editor_meta.edit_mode = !editor_meta.edit_mode;
                if editor_meta.edit_mode {
                    commands.trigger(UiRespawnTrigger::TileSelectionRespawn);
                } else {
                    commands.trigger(UiRespawnTrigger::TileSelectionRemove);
                }
            }
            EditorEvents::RespawnPlayer => {
                for (mut player, mut animation, parent, tile) in &mut players {
                    let Ok((mut velo, mut transform)) = parent_player.get_mut(parent.0) else {
                        continue;
                    };
                    tower_timer.timer = Some(Timer::from_seconds(0., TimerMode::Once));
                    *animation = PlayerAnimation::Idle;
                    player.mode = PlayerMode::Normal;
                    let translation = tile_to_world(
                        &tile.0.pos.into(),
                        TILEMAP_OFFSET.extend(LayerType::Entities.z() + 1.),
                    );
                    // this is just a hack because the respawn event sometimes fires 2 times
                    if translation.distance(transform.translation) > 1. {
                        editor_meta.deads += 1;
                        commands.trigger(UiRespawnTrigger::Deads);
                    }
                    commands
                        .entity(*cam)
                        .insert(ScreenShake::new(TRAUMA, 50., 1.));
                    transform.translation = translation;
                    **velo = Vec2::ZERO;
                }
                for (mut visibility, mut mode) in &mut enemies {
                    *visibility = Visibility::Inherited;
                    *mode = EnemyAnimation::Spawn;
                }
            }
        }
    }
}
fn update_control_images(
    mut commands: Commands,
    time: Res<Time>,
    mut control_images: Query<(Entity, &mut ControlImage)>,
) {
    let delta = time.delta();
    for (e, mut image) in &mut control_images {
        image.0.tick(delta);
        if image.0.finished() {
            commands.entity(e).despawn();
        }
    }
}
#[derive(Component)]
struct ControlImage(Timer);
fn show_control_image(
    trigger: Trigger<UiRespawnTrigger>,
    mut commands: Commands,
    textures: Res<map::Textures>,
    mut control_images: Query<&mut ControlImage>,
) {
    if !matches!(trigger.event(), UiRespawnTrigger::ShowControlImage) {
        return;
    }
    let timer = Timer::from_seconds(5., TimerMode::Once);
    if !control_images.is_empty() {
        for mut control_image in &mut control_images {
            control_image.0 = timer.clone();
        }
        return;
    }
    commands.spawn((
        Node {
            left: Val::Percent(70.),
            width: Val::Percent(30.),
            height: Val::Percent(10.),
            align_items: AlignItems::Center,
            top: Val::Percent(80.),
            ..default()
        },
        ControlImage(timer),
        // BackgroundColor(BLUE_500.into()),
        children![widget::tile_image(ImageNode::new(textures.keys.clone())),],
    ));
}
#[derive(Component)]
struct OverviewUiRoot;
fn init_ui_overview(
    trigger: Trigger<UiRespawnTrigger>,
    q: Query<Entity, With<OverviewUiRoot>>,
    mut commands: Commands,
    editor_meta: Res<EditorMeta>,
) {
    // cleanup in case of redrawing
    let UiRespawnTrigger::OverviewRespawn = trigger.event() else {
        return;
    };
    for e in &q {
        commands.entity(e).despawn();
    }
    let node = commands
        .spawn((
            Node {
                left: Val::Percent(20.),
                width: Val::Percent(60.),
                height: Val::Percent(4.),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(DEAD_BACKGROUND),
            OverviewUiRoot,
            children![
                widget::overview_button(
                    OverviewButton::EditMode,
                    if editor_meta.edit_mode {
                        "Edit mode"
                    } else {
                        "Play mode"
                    }
                ),
                widget::overview_button(OverviewButton::Load, "Load"),
            ],
        ))
        .id();
    commands
        .entity(node)
        .with_child(widget::overview_button(OverviewButton::Save, "Save"));

    if editor_meta.edit_mode {
        commands.entity(node).with_child(widget::overview_button(
            OverviewButton::LayerType,
            editor_meta.layer_type.name(),
        ));
    }
}

#[derive(Event)]
enum UiRespawnTrigger {
    TileSelectionRespawn,
    TileSelectionRemove,
    OverviewRespawn,
    ShowControlImage,
    Deads,
}
#[derive(Component)]
struct TileSelectionUiRoot;
fn ui_tile_selection_update(
    trigger: Trigger<UiRespawnTrigger>,
    editor_meta: Res<EditorMeta>,
    mut commands: Commands,
    q: Query<Entity, With<TileSelectionUiRoot>>,
    textures: Res<map::Textures>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    let event = trigger.event();
    if let UiRespawnTrigger::OverviewRespawn = event {
        return;
    }
    if let UiRespawnTrigger::ShowControlImage = event {
        return;
    }
    if let UiRespawnTrigger::Deads = event {
        return;
    }
    // cleanup in case of redrawing
    for e in &q {
        commands.entity(e).despawn();
    }
    if let UiRespawnTrigger::TileSelectionRemove = event {
        return;
    }
    commands
        .spawn((
            widget::tile_selection_root(Val::Percent(88.)),
            TileSelectionUiRoot,
        ))
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
                    let textures = &textures.pack[&editor_meta.layer_type];
                    let atlas = texture_atlas_layouts.get(textures.layout.id()).unwrap();
                    parent.spawn((
                        widget::tile_container(Val::Percent(11.)),
                        children![widget::tile_image(ImageNode::solid_color(
                            palettes::basic::WHITE.into()
                        ))],
                    ));
                    for i in 0..atlas.len() / 2 {
                        parent.spawn((
                            widget::tile_container(Val::Percent(10.)),
                            children![
                                widget::tile_image(ImageNode::from_atlas_image(
                                    textures.texture.clone(),
                                    TextureAtlas {
                                        layout: textures.layout.clone(),
                                        index: i * 2,
                                    },
                                )),
                                widget::tile_image(ImageNode::from_atlas_image(
                                    textures.texture.clone(),
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

#[derive(Component)]
pub struct TileButton;
#[derive(Component)]
pub enum OverviewButton {
    EditMode,
    LayerType,
    Save,
    Load,
}
fn overview_button_system(
    mut commands: Commands,
    mut layer_node_q: Query<
        (&Interaction, &mut Text, &mut Outline, &OverviewButton),
        (With<Button>, Changed<Interaction>),
    >,
    mut editor_meta: ResMut<EditorMeta>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<EditorEvents>,
) {
    for (interaction, mut text, mut outline, overview_button) in &mut layer_node_q {
        match *interaction {
            Interaction::Pressed => match overview_button {
                OverviewButton::LayerType => {
                    editor_meta.layer_type = editor_meta.layer_type.next();
                    **text = editor_meta.layer_type.name().into();
                    commands.trigger(UiRespawnTrigger::TileSelectionRespawn);
                }
                OverviewButton::Save => {
                    event_writer.write(EditorEvents::SaveLevel);
                }
                OverviewButton::Load => {
                    let mut name = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                        Some("level0".into())
                    } else {
                        None
                    };
                    if cfg!(target_arch = "wasm32") {
                        name = Some("level0".into());
                    }
                    editor_meta.current_level_index = 0;
                    event_writer.write(EditorEvents::LoadLevel { name });
                }
                OverviewButton::EditMode => {
                    event_writer.write(EditorEvents::ToggleEditMode);
                    **text = if editor_meta.edit_mode {
                        "Play mode"
                    } else {
                        "Edit mode"
                    }
                    .into();
                }
            },
            Interaction::Hovered => match overview_button {
                OverviewButton::LayerType => {
                    text.push_str("  ");
                    text.push_str(editor_meta.layer_type.next().name());
                    outline.color = HOVERED_BUTTON;
                }
                OverviewButton::Save | OverviewButton::Load | OverviewButton::EditMode => {
                    outline.color = HOVERED_BUTTON;
                }
            },
            Interaction::None => match overview_button {
                OverviewButton::LayerType => {
                    outline.color = NORMAL_BUTTON;
                    **text = editor_meta.layer_type.name().into();
                }
                OverviewButton::Save | OverviewButton::Load | OverviewButton::EditMode => {
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
            if editor_meta.selected_tile != image_node.texture_atlas
                && outline.color == PRESSED_BUTTON
            {
                outline.color = NORMAL_BUTTON;
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
                // the texture atlas is always of variant `Option::Some` or is the eraser
                editor_meta.selected_tile = layout.texture_atlas.clone();
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
fn debug(
    mut camera: Single<&mut Camera, With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<&Sprite, With<Player>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        for sprite in &players {
            let atlas = atlases
                .get(sprite.texture_atlas.as_ref().unwrap().layout.id())
                .unwrap();
            println!("{:?}", &sprite.texture_atlas);
            println!("{:?}", &atlas);
        }
    }
    if keys.just_pressed(KeyCode::KeyT) {
        if let ClearColorConfig::Custom(current) = camera.clear_color {
            if current == BACKGROUND_COLOR {
                camera.clear_color = ClearColorConfig::Custom(DEBUG_BACKGROUND_COLOR);
            } else {
                camera.clear_color = ClearColorConfig::Custom(BACKGROUND_COLOR);
            }
        }
    }
}
fn check_level_loaded(
    editor_meta: Res<EditorMeta>,
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<EditorEvents>,
) {
    let state = asset_server.get_load_state(editor_meta.current_level.id());
    if let Some(LoadState::Failed(failed)) = state {
        warn!("failed loading level {failed:?}");
        event_writer.write(EditorEvents::LoadLevel {
            name: Some("level0".into()),
        });
    }
}
