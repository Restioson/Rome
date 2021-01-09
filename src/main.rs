use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use crate::map::shader::MapMaterial;
use crate::loading::LoadRomeAssets;
use crate::map::RomeMapPlugin;
use bevy::prelude::shape::Cube;
use bevy::render::render_graph::base::MainPass;
use goshawk::{RtsCamera, ZoomSettings, PanSettings, TurnSettings};
use bevy::render::pipeline::{PipelineDescriptor, RenderPipeline};
use bevy::render::shader::{ShaderStage, ShaderStages};

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
    normal_mesh: Handle<Mesh>,
}

fn fps_counter_text_update(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        if let (Some(fps), Some(frame_time)) = (diagnostics.get(FrameTimeDiagnosticsPlugin::FPS), diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)) {
            if let (Some(average_fps), Some(average_frame_time)) = (fps.average(), frame_time.average()) {
                text.value = format!("FPS: {:.0} Frame time: {:.2}ms", average_fps.round(), average_frame_time * 1000.0).into();
            }
        }
    }
}

fn start_game(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    assets: Res<RomeAssets>,
    asset_server: ResMut<AssetServer>
) {
    // let italy = Vec3::new(599.0, 0.0, 440.0);
    let italy = Vec3::new(0.0, 0.0, 0.0);
    let font_handle = asset_server.load("fonts/FiraSans-SemiBold.ttf");


    const VERTEX_SHADER: &str = "
        #version 450

        layout(location = 0) in vec3 Vertex_Position;

        layout(set = 2, binding = 4) uniform utexture2D MapMaterial_heightmap;
        layout(set = 2, binding = 5) uniform sampler MapMaterial_heightmap_sampler;

        layout(set = 0, binding = 0) uniform Camera {
            mat4 ViewProj;
        };

        layout(set = 1, binding = 0) uniform Transform {
            mat4 Model;
        };

        float height(ivec2 pos) {
            uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), pos, 0).r;
            uint height = packed >> 8;
            return height;
        }

        void main() {
            vec3 p = Vertex_Position;
            float avg_height = (
                height(ivec2(floor(p.x), floor(p.z))) +
                height(ivec2(ceil(p.x), ceil(p.z))) +
                height(ivec2(floor(p.x), ceil(p.z))) +
                height(ivec2(ceil(p.x), floor(p.z)))
            ) / 4.0;
            p.y += avg_height * 0.1;

            gl_Position = ViewProj * Model * vec4(p, 1.0);
        }
        ";
    const FRAGMENT_SHADER: &str = "
            #version 450
            layout(location = 0) out vec4 o_Target;

            void main() {
                o_Target = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ";

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    commands.spawn(Camera3dBundle::default())
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
        .with_children(|builder| {
            builder.spawn(MeshBundle {
                    mesh: assets.normal_mesh.clone(),
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(pipeline_handle)]),
                    ..Default::default()
                }
            ).with(assets.map_material.clone())
            .spawn(MeshBundle {
                mesh: assets.clipmap_mesh.clone(),
                render_pipelines: map::shader::render_pipelines(),
                transform: Transform::from_translation(Vec3::default()),
                ..Default::default()
            })
            .with(assets.map_material.clone());
        });

    // TODO
        // .with(TurnSettings {
        //     mouse_turn_margin: 0.0,
        //     max_speed: 0.0, // disable turning
        //     ..Default::default()
        // });

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
