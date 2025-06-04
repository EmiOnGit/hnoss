use std::collections::VecDeque;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<AssetLoadingQueue>()
        .add_systems(PreUpdate, check_assets);
}
#[derive(Resource, Default)]
pub struct AssetLoadingQueue {
    loading: VecDeque<UntypedHandle>,
    finished: Vec<UntypedHandle>,
}
pub trait LoadResource {
    fn load_resource<T: Asset + FromWorld>(&mut self) -> &mut Self;
}
impl LoadResource for App {
    fn load_resource<T: Asset + FromWorld>(&mut self) -> &mut Self {
        self.init_asset::<T>();
        let world = self.world_mut();
        let asset = T::from_world(world);
        let asset_server = world.resource::<AssetServer>();
        let handle = asset_server.add(asset);
        let mut loading_queue = world.resource_mut::<AssetLoadingQueue>();
        loading_queue.loading.push_back(handle.untyped());
        self
    }
}
fn check_assets(asset_server: ResMut<AssetServer>, mut loading_queue: ResMut<AssetLoadingQueue>) {
    for _ in 0..loading_queue.loading.len() {
        let Some(loading) = loading_queue.loading.pop_front() else {
            break;
        };
        if asset_server.is_loaded_with_dependencies(loading.id()) {
            println!("loaded");
            loading_queue.finished.push(loading);
        } else {
            println!("not yet loaded");
            loading_queue.loading.push_back(loading);
        }
    }
}
pub fn all_assets_loaded(loading_queue: Res<AssetLoadingQueue>) -> bool {
    loading_queue.loading.is_empty()
}
