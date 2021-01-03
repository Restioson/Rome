use crate::map::shader::MapMaterial;
use crate::map::HeightMap;
use crate::map::{mesh::build_mesh, HeightMapLoader};
use crate::{AppState, RomeAssets, STATE_STAGE};
use bevy::prelude::*;
use bevy::render::texture::{
    AddressMode, Extent3d, SamplerDescriptor, TextureDimension, TextureFormat,
};
use bevy::tasks::AsyncComputeTaskPool;
use itertools::Itertools;

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
}

/// Assets loaded from disk
struct LoadedAssets {
    forest: Handle<Texture>,
    sand: Handle<Texture>,
    heightmap: Handle<Texture>,
}

impl LoadingAssets {
    fn all_loaded(&self) -> Option<LoadedAssets> {
        match (
            self.forest.clone(),
            self.sand.clone(),
            self.heightmap.clone(),
        ) {
            (Some(forest), Some(sand), Some(heightmap)) => Some(LoadedAssets {
                forest,
                sand,
                heightmap,
            }),
            _ => None,
        }
    }
}

fn queue_asset_load(asset_server: Res<AssetServer>) {
    asset_server.load_folder("map/heightmap").unwrap();
    asset_server.load_folder("map/textures").unwrap();
}

fn loading(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Texture>>,
    heightmaps: Res<Assets<HeightMap>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut loading: ResMut<LoadingAssets>,
    mut state: ResMut<State<AppState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    fn set_address_mode(s: &mut SamplerDescriptor, m: AddressMode) {
        s.address_mode_u = m;
        s.address_mode_v = m;
        s.address_mode_w = m;
    }

    let sand_handle = asset_server.load("map/textures/beach_sand.png");
    if let Some(tx) = textures
        .get_mut(&sand_handle)
        .filter(|_| loading.sand.is_none())
    {
        set_address_mode(&mut tx.sampler, AddressMode::Repeat);
        loading.sand = Some(sand_handle);
    }

    let forest_handle = asset_server.load("map/textures/forest2.png");
    if let Some(tx) = textures
        .get_mut(&forest_handle)
        .filter(|_| loading.forest.is_none())
    {
        // set_address_mode(&mut tx.sampler, AddressMode::Repeat);
        loading.forest = Some(forest_handle);
    }

    if let Some(map) = heightmaps
        .get("map/heightmap/map.mapdat")
        .filter(|_| loading.heightmap.is_none())
    {
        let mut bytes = Vec::with_capacity(1024 * 1024 * 2);

        for (x, z) in (0..1024).cartesian_product(0..1024) {
            let height = map.0.get(x, z).height;
            bytes.push((height.0 >> 8) as u8);
            bytes.push((height.0 & 0xff) as u8);
        }

        let texture_map = Texture {
            data: bytes,
            size: Extent3d::new(1024, 1024, 1),
            format: TextureFormat::Rgba32Float,
            dimension: TextureDimension::D2,
            sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..Default::default()
            },
        };

        loading.heightmap = Some(textures.add(texture_map));
    }

    if let Some(LoadedAssets {
        forest,
        sand,
        heightmap,
    }) = loading.all_loaded()
    {
        dbg!(&forest.is_strong());
        let map_material = materials.add(MapMaterial { forest });
        dbg!(&map_material.is_strong());
        let clipmap_mesh = meshes.add(build_mesh(4)); // TODO in task pool
        commands.insert_resource(RomeAssets {
            map_material,
            clipmap_mesh,
        });

        state.set_next(AppState::InGame).unwrap();
        dbg!("Done loading");
        // TODO remove loading_state resource
    }
}
