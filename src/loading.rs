use crate::map::shader::MapMaterial;
use crate::map::HeightMap;
use crate::map::{mesh::build_mesh, HeightMapLoader};
use crate::{AppState, RomeAssets, STATE_STAGE};
use bevy::prelude::*;
use bevy::render::texture::{AddressMode, Extent3d, SamplerDescriptor, TextureDimension, TextureFormat, FilterMode};
use bevy::tasks::AsyncComputeTaskPool;
use itertools::Itertools;
use byteorder::{WriteBytesExt, NativeEndian};

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
            self.forest.as_ref(),
            self.sand.as_ref(),
            self.heightmap.as_ref(),
        ) {
            (Some(forest), Some(sand), Some(heightmap)) => Some(LoadedAssets {
                forest: forest.clone(),
                sand: sand.clone(),
                heightmap: heightmap.clone(),
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
    heightmaps: Res<Assets<HeightMap>>,
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

    let forest_handle = asset_server.load::<Texture, &str>("map/textures/forest2.png");
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
        let mut bytes = Vec::with_capacity(1024 * 1024 * 2);

        for (x, z) in (0..1024).cartesian_product(0..1024) {
            let px = map.0.get(x, z);

            bytes.write_i16::<NativeEndian>(if px.is_water { 0 } else { px.height.0 }).unwrap();
        }

        let texture_map = Texture {
            data: bytes,
            size: Extent3d::new(1024, 1024, 1),
            format: TextureFormat::R16Sint,
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
        let map_material = materials.add(MapMaterial { forest, sand, heightmap });
        let clipmap_mesh = meshes.add(build_mesh(4)); // TODO in task pool
        commands.insert_resource(RomeAssets {
            map_material,
            clipmap_mesh,
        });

        state.set_next(AppState::InGame).unwrap();
        // TODO remove loading_state resource
    }
}
