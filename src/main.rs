use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use crate::map::shader::MapMaterial;
use crate::loading::LoadRomeAssets;
use crate::map::{RomeMapPlugin, LIGHT_POS, LatLong};
use goshawk::{RtsCamera, ZoomSettings, PanSettings, TurnSettings};
use bevy::prelude::shape::Cube;

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

fn fps_counter_text_update(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>, query2: Query<&goshawk::RtsCamera>) {
    let xyz = query2.iter().next().map(|opt| opt.looking_at);
    let dist = query2.iter().next().map(|opt| opt.zoom_distance);
    for mut text in query.iter_mut() {
        if let (Some(fps), Some(frame_time)) = (diagnostics.get(FrameTimeDiagnosticsPlugin::FPS), diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)) {
            if let (Some(average_fps), Some(average_frame_time), Some(xyz), Some(dist)) = (fps.average(), frame_time.average(), xyz, dist) {
                text.value = format!("FPS: {:.0}. Frame time: {:.2}ms. XZ: ({:.2}; {:.2}). Zoom: {:.2}.", average_fps.round(), average_frame_time * 1000.0, xyz.x, xyz.z, dist).into();
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
    let italy = Vec3::new(745.0, 0.0, 535.0);
    let rome = LatLong {
        latitude: 41.9,
        longitude: 12.49
    }.to_tile_coord().to_world_space_0y();

    let font_handle = asset_server.load("fonts/FiraMono-Medium.ttf");
    let light_pos: Vec3 = LIGHT_POS.into();

    commands.spawn(Camera3dBundle::default())
        .with(RtsCamera {
            looking_at: italy,
            zoom_distance: 175.0,
            ..Default::default()
        })
        .with(ZoomSettings {
            angle_range: 0.9103..=1.237539,
            scroll_accel: 25.0,
            max_velocity: 70.0,
            idle_deceleration: 200.0,
            angle_change_zone: 80.0..=130.0,
            distance_range: 75.0..=200.0,
            ..Default::default()
        })
        .with(PanSettings {
            mouse_accel: 200.0,
            keyboard_accel: 160.0,
            idle_deceleration: 200.0,
            max_speed: 10.0,
            pan_speed_zoom_factor_range: 1.0..=6.0,
            ..Default::default()
        })
        .with(TurnSettings {
            mouse_turn_margin: 0.0,
            mouse_accel: 0.0,
            keyboard_accel: 0.0,
            ..Default::default()
        })
        .spawn(MeshBundle {
            mesh: assets.clipmap_mesh.clone(),
            render_pipelines: map::shader::render_pipelines(),
            transform: Transform::from_translation(Vec3::splat(0.5)),
            ..Default::default()
        })
        .with(assets.map_material.clone())
        .spawn(PbrBundle {
            mesh: meshes.add(Cube::new(5.0).into()),
            material: materials.add(StandardMaterial::default()),
            transform: Transform::from_translation(rome),
            ..Default::default()
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(light_pos * 1_000_000.0),
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
