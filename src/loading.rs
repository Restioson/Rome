use crate::map::shader::{MapMaterial};
use crate::map::HeightMap;
use crate::map::{mesh::build_mesh, HeightMapLoader};
use crate::{AppState, RomeAssets, STATE_STAGE};
use bevy::prelude::*;
use bevy::render::texture::{AddressMode, SamplerDescriptor, FilterMode};
use bevy::tasks::AsyncComputeTaskPool;
use crate::map::mipmap::generate_mipmaps;
use std::time::Instant;

pub struct LoadRomeAssets;

impl Plugin for LoadRomeAssets {
    fn build(&self, app: &mut AppBuilder) {
        let task_pool = (*app.resources().get::<AsyncComputeTaskPool>().unwrap()).clone();

        app
            .add_resource(LoadingAssets::default())
            .add_asset_loader(HeightMapLoader { task_pool })
            .add_startup_system(queue_asset_load.system())
            .on_state_update(STATE_STAGE, AppState::Loading, loading.system());
    }
}

/// Assets loading from disk
#[derive(Default)]
struct LoadingAssets {
    forest: Option<Handle<Texture>>,
    sand: Option<Handle<Texture>>,
    heightmap: Option<Handle<Texture>>,
    raw_heightmap: Option<(Handle<HeightMap>, u16)>,
}

/// Assets loaded from disk
struct LoadedAssets {
    forest: Handle<Texture>,
    sand: Handle<Texture>,
    heightmap: Handle<Texture>,
    raw_heightmap: Handle<HeightMap>,
    max_y: u16,
}

impl LoadingAssets {
    fn all_loaded(&self) -> Option<LoadedAssets> {
        match (
            self.forest.as_ref(),
            self.sand.as_ref(),
            self.heightmap.as_ref(),
            self.raw_heightmap.as_ref(),
        ) {
            (Some(forest), Some(sand), Some(heightmap), Some(raw_heightmap)) => Some(LoadedAssets {
                forest: Handle::clone(forest),
                sand: Handle::clone(sand),
                heightmap: Handle::clone(heightmap),
                raw_heightmap: Handle::clone(&raw_heightmap.0),
                max_y: raw_heightmap.1,
            }),
            _ => None,
        }
    }
}

fn queue_asset_load(asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
    asset_server.load_folder("map/heightmap").unwrap();
    asset_server.load_folder("map/textures").unwrap();
}

fn loading(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut heightmaps: ResMut<Assets<HeightMap>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut loading: ResMut<LoadingAssets>,
    mut state: ResMut<State<AppState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    fn setup_texture(s: &mut SamplerDescriptor) {
        s.address_mode_u = AddressMode::Repeat;
        s.address_mode_v = AddressMode::Repeat;
        s.address_mode_w = AddressMode::Repeat;
        s.mipmap_filter = FilterMode::Linear;
    }

    let sand_handle = asset_server.load::<Texture, &str>("map/textures/beach_sand.png");
    if let Some(tx) = textures
        .get_mut(&sand_handle)
        .filter(|_| loading.sand.is_none())
    {
        setup_texture(&mut tx.sampler);
        let tx = tx.clone();
        // TODO: For some reason this is required, else the texture will be dropped early (???)
        loading.sand = Some(textures.add(tx));
    }

    let forest_handle = asset_server.load::<Texture, &str>("map/textures/grassland.png");
    if let Some(tx) = textures
        .get_mut(&forest_handle)
        .filter(|_| loading.forest.is_none())
    {
        setup_texture(&mut tx.sampler);
        let tx = tx.clone();
        loading.forest = Some(textures.add(tx));
    }

    if let Some(map) = heightmaps
        .get("map/heightmap/map.mapdat")
        .filter(|_| loading.heightmap.is_none())
    {
        // TODO in task pool
        let (texture, max_y) = time("Generating heightmap texture", || map.into());
        loading.heightmap = Some(textures.add(texture));
        let cloned = map.clone();

        loading.raw_heightmap = Some((heightmaps.add(cloned), max_y));
    }

    if let Some(LoadedAssets {
        forest,
        sand,
        heightmap,
        raw_heightmap,
        max_y
    }) = loading.all_loaded()
    {
        let mipmaps = time("Generating mipmap", move || generate_mipmaps(&heightmaps.get(raw_heightmap).unwrap().0, 1));
        let mipmap = &mipmaps[0];
        let tx = time("Converting mipmap to texture", || mipmap.to_texture(max_y));
        let map_material = materials.add(
            MapMaterial {
                forest, 
                sand, 
                heightmap,
                mipmap: textures.add(tx),
            }
        );
        let clipmap_mesh = meshes.add(time("Building clipmap mesh", || build_mesh(6))); // TODO in task pool
        commands.insert_resource(RomeAssets { map_material, clipmap_mesh });

        state.set_next(AppState::InGame).unwrap();
        // TODO remove loading_state resource
    }
}

pub fn time<F: FnOnce() -> T, T>(label: &str, f: F) -> T {
    let now = Instant::now();
    let ret = f();
    eprintln!("{} took {:.2}s", label, now.elapsed().as_secs_f32());
    ret
}
