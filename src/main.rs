use crate::map::MapGenerator;
use bevy::prelude::*;
use bevy::window::WindowMode;

mod map;
mod rts_camera;

use rts_camera::rts_camera_system;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(rts_camera_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let generator = MapGenerator::new();
    let mesh_handles = generator.generate_meshes(&mut meshes);

    let mut translation = Vec3::new(0.0, 0.0, 0.0);
    for ((x, z), mesh) in mesh_handles {
        translation = Vec3::new(x as f32, 0.0, z as f32);
        commands.spawn(PbrComponents {
            mesh,
            material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
            transform: Transform::from_translation(translation),
            ..Default::default()
        });
    }

    let italy = Vec3::new(translation.x() / 2.2, 0.0, translation.z() / 1.75);
    // 45 degrees
    let camera_state =
        rts_camera::State::new_looking_at_zoomed_out(italy, std::f32::consts::PI / 4.0, 128.0);
    let camera_transform = camera_state.camera_transform();

    commands
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 128.0, translation.z() / 2.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: camera_transform,
            ..Default::default()
        })
        .with(camera_state);
}
