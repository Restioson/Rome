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
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics};
use once_cell::sync::Lazy;
use crate::map::shader::MapMaterial;
use bevy::render::texture::AddressMode;

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
        .add_asset::<MapMaterial>()
        .add_startup_system(map::shader::setup.system())
        .add_startup_system(setup.system())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(fps_counter_text_update.system())
        .add_system(rts_camera_system.system())
        .add_system(map::shader::update_time.system())
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
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new("([0-9]+)x([0-9]+)\\.heightmap").unwrap());

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

fn fps_counter_text_update(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    for mut text in &mut query.iter() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.value = format!("FPS: {:.0}", average.round()).into();
            }
        }
    }
}

struct MeshData {
    texture: Handle<Texture>,
}

fn spawn_meshes(
    mut commands: Commands,
    mesh_data: Res<MeshData>,
    generator: Res<MapGenerator>,
    mut heightmap_asset_events: ResMut<Events<AssetEvent<HeightMap>>>,
    heightmaps: Res<Assets<HeightMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapMaterial>>,
) {
    for event in heightmap_asset_events.drain() {
        let heightmap = match event {
            AssetEvent::Created { handle } => heightmaps.get(&handle).unwrap(),
            _ => unimplemented!(),
        };

        let material = MapMaterial { texture: mesh_data.texture };
        let material = materials.add(material);

        let meshes = generator.generate_meshes(heightmap, &mut meshes);

        for ((x, z), mesh) in meshes {
            let translation = Vec3::new(
                x as f32,
                0.0,
                z as f32
            );

            commands
                .spawn(MeshComponents {
                    mesh,
                    render_pipelines: map::shader::render_pipelines(),
                    transform: Transform::from_translation(translation),
                    ..Default::default()
                })
                .with(material)
                .with(map::shader::TimeNode::default());
        }
    }
}

fn setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: Res<AssetServer>,
) {
    asset_server.load_asset_folder("assets/heightmap").unwrap();
    asset_server.watch_for_changes().unwrap();
    let italy = Vec3::new(413.0, 0.0, 437.0);
    let angle = std::f32::consts::PI / 4.0;
    let camera_state = rts_camera::State::new_looking_at_zoomed_out(italy, angle, 180.0);
    let camera_transform = camera_state.camera_transform();
    let font_handle = asset_server.load("assets/fonts/FiraSans-SemiBold.ttf").unwrap();

    let texture = asset_server.load_sync(&mut textures, "assets/texture2.png").unwrap();
    textures.get_mut(&texture).unwrap().address_mode = AddressMode::Repeat;

    commands
        .insert_resource(MeshData { texture })
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 180.0, 437.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: camera_transform,
            ..Default::default()
        })
        .with(camera_state)
        .spawn(UiCameraComponents::default())
        .spawn(TextComponents {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "FPS:".to_string(),
                font: font_handle,
                style: TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            },
            ..Default::default()
        });
}
