use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use crate::map::shader::MapMaterial;
use crate::loading::LoadRomeAssets;
use crate::map::RomeMapPlugin;
use crate::rts_camera::rts_camera_system;
use bevy::prelude::shape::Cube;
use crate::map::mesh::build_mesh;
use bevy::render::camera::VisibleEntities;

mod loading;
mod map;
mod rts_camera;

const STATE_STAGE: &'static str = "rome_app_state_stage";

fn main() {
    let mut builder = App::build();

    builder
        .add_resource(Msaa { samples: 8 })
        .add_resource(WindowDescriptor {
            vsync: false,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_resource(State::new(AppState::Loading))
        .add_stage_after(stage::UPDATE, STATE_STAGE, StateStage::<AppState>::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LoadRomeAssets)
        .add_plugin(RomeMapPlugin)
        .on_state_enter(STATE_STAGE, AppState::InGame, start_game.system())
        .on_state_update(
            STATE_STAGE,
            AppState::InGame,
            fps_counter_text_update.system(),
        )
        .on_state_update(STATE_STAGE, AppState::InGame, rts_camera_system.system())
        .run();
}

#[derive(Clone)]
enum AppState {
    Loading,
    InGame,
}

pub struct RomeAssets {
    map_material: Handle<MapMaterial>,
    clipmap_mesh: Handle<Mesh>,
}

fn fps_counter_text_update(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.value = format!("FPS: {:.0}", average.round()).into();
            }
        }
    }
}

fn start_game(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<RomeAssets>,
    asset_server: ResMut<AssetServer>
) {
    dbg!("Started game");

    // let italy = Vec3::new(599.0, 0.0, 440.0);
    let italy = Vec3::new(0.0, 0.0, 0.0);
    let angle = std::f32::consts::PI / 4.0;
    let camera_state = rts_camera::RtsCamera::new_looking_at_zoomed_out(italy, angle, 180.0);
    let camera_transform = camera_state.camera_transform();
    let font_handle = asset_server.load("fonts/FiraSans-SemiBold.ttf");

    commands
        .spawn(MeshBundle {
            mesh: assets.clipmap_mesh.clone(),
            render_pipelines: map::shader::render_pipelines(),
            transform: Transform::from_translation(Vec3::default()),
            ..Default::default()
        })
        .with(assets.map_material.clone())
        .spawn(Camera3dBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .with(camera_state);

    // debug cube
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cube::new(10.0))),
            material: materials.add(StandardMaterial::default()),
            ..Default::default()
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 180.0, 437.0)),
            ..Default::default()
        })
        .spawn(CameraUiBundle::default())
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "FPS:".to_string(),
                font: font_handle,
                style: TextStyle {
                    font_size: 40.0,
                    alignment: TextAlignment::default(),
                    color: Color::WHITE,
                },
            },
            ..Default::default()
        });
}
