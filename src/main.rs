use crate::map::mesh::MapGenerator;
use bevy::prelude::*;
use bevy::window::WindowMode;

mod map;
mod rts_camera;

use rts_camera::rts_camera_system;
use bevy::asset::AssetLoader;
use std::path::Path;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics};
use crate::map::shader::MapMaterial;
use bevy::render::texture::AddressMode;
use std::io::Read;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 8 })
        .add_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
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
        .run();
}

pub struct HeightMap(rome_map::Map);

#[derive(Default)]
struct HeightMapLoader;

impl AssetLoader<HeightMap> for HeightMapLoader {
    fn from_bytes(&self, _path: &Path, bytes: Vec<u8>) -> Result<HeightMap, anyhow::Error> {
        println!("Unzipping");

        let mut decoder = zstd::Decoder::new(&*bytes).unwrap();
        let mut unzipped = Vec::new();
        decoder.read_to_end(&mut unzipped).unwrap();

        println!("Deserializing");
        let map: rome_map::Map = bincode::deserialize(&unzipped).unwrap();
        println!("Done loading heightmap");

        Ok(HeightMap(map))
    }

    fn extensions(&self) -> &[&str] {
        &["mapdat"]
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

fn setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut heightmap: ResMut<Assets<HeightMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let map_handle = asset_server.load_sync(&mut heightmap, "assets/map/heightmap/map.mapdat").unwrap();
    let map = heightmap.get(&map_handle).unwrap();

    let generator = MapGenerator::new();
    let meshes = generator.generate_meshes(&map.0, &mut meshes);

    let forest_texture = asset_server.load_sync(&mut textures, "assets/map/textures/forest2.png").unwrap();
    textures.get_mut(&forest_texture).unwrap().address_mode = AddressMode::Repeat;

    let beach_texture = asset_server.load_sync(&mut textures, "assets/map/textures/beach_sand.png").unwrap();
    textures.get_mut(&beach_texture).unwrap().address_mode = AddressMode::Repeat;

    let map_material = MapMaterial { forest_texture, beach_texture };
    let map_material = materials.add(map_material);

    for ((x, z), mesh) in meshes {
        let translation = Vec3::new(x as f32, 0.0, z as f32);

        commands
            .spawn(MeshComponents {
                mesh,
                render_pipelines: map::shader::render_pipelines(),
                transform: Transform::from_translation(translation),
                ..Default::default()
            })
            .with(map_material.clone())
            .with(map::shader::TimeNode::default());
    }

    asset_server.watch_for_changes().unwrap();
    let italy = Vec3::new(599.0, 0.0, 440.0);
    let angle = std::f32::consts::PI / 4.0;
    let camera_state = rts_camera::State::new_looking_at_zoomed_out(italy, angle, 180.0);
    let camera_transform = camera_state.camera_transform();
    let font_handle = asset_server.load("assets/fonts/FiraSans-SemiBold.ttf").unwrap();

    commands
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
