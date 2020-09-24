use bevy::prelude::*;
use bevy::window::WindowMode;
use crate::map::HeightmapSampler;

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
        // .add_plugin(bevy_fly_camera::FlyCameraPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {

    let sampler = HeightmapSampler::new();
    let mesh = meshes.add(sampler.create_mesh());

    commands
        .spawn(PbrComponents {
            mesh,
            material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
            transform: Transform::from_translation(Vec3::new(-1.5, 0.0, 0.0)),
            ..Default::default()
        })
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 75.0, 256.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: Transform::new(Mat4::face_toward(
                Vec3::new(128.0, 100.0, 256.0 + 64.0),
                Vec3::new(128.0, 0.0, 128.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with(rts_camera::State::default());
        // .with(bevy_fly_camera::FlyCamera::default());
}
