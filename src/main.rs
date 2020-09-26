use crate::map::mesh::MapGenerator;
use bevy::prelude::*;
use bevy::window::WindowMode;

mod map;
mod rts_camera;

use rts_camera::rts_camera_system;
use bevy::asset::AssetLoader;
use std::path::Path;
use crate::map::terrarium_raster::Raster;
use regex::Regex;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 8 })
        .add_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_resource(MapGenerator::new())
        .add_default_plugins()
        .add_asset::<HeightMap>()
        .add_asset_loader::<HeightMap, HeightMapLoader>()
        .add_startup_system(setup.system())
        .add_system(rts_camera_system.system())
        .add_system(spawn_meshes.system())
        .run();
}

pub struct HeightMap {
    grid_pos: (u32, u32),
    raster: Raster,
}

#[derive(Default)]
struct HeightMapLoader;

impl AssetLoader<HeightMap> for HeightMapLoader {
    fn from_bytes(&self, path: &Path, bytes: Vec<u8>) -> Result<HeightMap, anyhow::Error> {
        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new("([0-9]+)x([0-9]+)\\.heightmap").unwrap();
        }

        let filename = path.file_name().unwrap().to_str().unwrap();
        let captures = RE.captures(filename).unwrap();
        let get_coord = |idx| captures.get(idx + 1usize).unwrap().as_str().parse::<u32>().unwrap();

        Ok(HeightMap {
            grid_pos: (get_coord(0), get_coord(1)),
            raster: map::terrarium_raster::read(&*bytes).unwrap(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["heightmap"]
    }
}

fn spawn_meshes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    generator: Res<MapGenerator>,
    mut heightmap_asset_events: ResMut<Events<AssetEvent<HeightMap>>>,
    heightmaps: Res<Assets<HeightMap>>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in heightmap_asset_events.drain() {
        let heightmap = match event {
            AssetEvent::Created { handle } => heightmaps.get(&handle).unwrap(),
            _ => unimplemented!(),
        };

        let meshes = generator.generate_meshes(heightmap, &mut meshes);

        let texture: Handle<Texture> = asset_server.load_sync(
            &mut textures,
            "assets/texture.png"
        ).unwrap();

        let material = materials.add(texture.into());
        for ((x, z), mesh) in meshes {
            let translation = Vec3::new(
                x as f32,
                0.0,
                z as f32
            );

            commands.spawn(PbrComponents {
                mesh,
                material,
                transform: Transform::from_translation(translation),
                ..Default::default()
            });
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    asset_server.load_asset_folder("assets/heightmap").unwrap();
    asset_server.watch_for_changes().unwrap();
    let italy = Vec3::new(413.0, 0.0, 437.0);
    let angle = std::f32::consts::PI / 4.0;
    let camera_state = rts_camera::State::new_looking_at_zoomed_out(italy, angle, 180.0);
    let camera_transform = camera_state.camera_transform();

    commands
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 180.0, 437.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: camera_transform,
            ..Default::default()
        })
        .with(camera_state);
}
