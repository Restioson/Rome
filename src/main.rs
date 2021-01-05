use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use crate::map::shader::MapMaterial;
use crate::loading::LoadRomeAssets;
use crate::map::RomeMapPlugin;
use bevy::prelude::shape::Cube;
use bevy::render::render_graph::base::MainPass;
use goshawk::{RtsCamera, ZoomSettings, PanSettings, TurnSettings};

mod loading;
mod map;

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
        .add_system(fps_counter_text_update.system())
        .on_state_enter(STATE_STAGE, AppState::InGame, start_game.system())
        .on_state_update(STATE_STAGE, AppState::InGame, goshawk::rts_camera_system.system())
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
    // let italy = Vec3::new(599.0, 0.0, 440.0);
    let italy = Vec3::new(0.0, 0.0, 0.0);
    let font_handle = asset_server.load("fonts/FiraSans-SemiBold.ttf");

    commands
        .spawn(MeshBundle {
            mesh: assets.clipmap_mesh.clone(),
            render_pipelines: map::shader::render_pipelines(),
            transform: Transform::from_translation(Vec3::default()),
            ..Default::default()
        })
        .with(MainPass)
        .with(assets.map_material.clone())
        .spawn(Camera3dBundle::default())
        .with(RtsCamera {
            looking_at: italy,
            zoom_distance: 100.0,
            ..Default::default()
        })
        .with(ZoomSettings {
            scroll_accel: 20.0,
            max_velocity: 80.0,
            idle_deceleration: 400.0,
            angle_change_zone: 75.0..=200.0,
            distance_range: 50.0..=300.0,
            ..Default::default()
        })
        .with(PanSettings {
            mouse_accel: 50.0,
            keyboard_accel: 40.0,
            idle_deceleration: 50.0,
            max_speed: 20.0,
            pan_speed_zoom_factor_range: 1.0..=4.0,
            ..Default::default()
        })
        .with(TurnSettings {
            mouse_turn_margin: 0.0,
            max_speed: 0.0, // disable turning
            ..Default::default()
        });

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
