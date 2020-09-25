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
    meshes: ResMut<Assets<Mesh>>,
) {
    let generator = MapGenerator::new();
    let mesh_handles = generator.generate_meshes(meshes);

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

    // Centred on italy
    let camera_translation = Vec3::new(translation.x() / 2.35, 128.0, translation.z() / 1.75);
    let camera_rotation = Quat::from_rotation_x(-45.0);
    let camera_transform = Mat4::from_rotation_translation(camera_rotation, camera_translation);

    commands
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 128.0, translation.z() / 2.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: Transform::new(camera_transform),
            ..Default::default()
        })
        .with(rts_camera::State::default());
}
