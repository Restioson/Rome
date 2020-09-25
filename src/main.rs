use crate::map::mesh::MapGenerator;
use crate::map::shader::{MyMaterial, VERTEX_SHADER, FRAGMENT_SHADER};
use bevy::prelude::*;
use bevy::window::WindowMode;

mod map;
mod rts_camera;

use rts_camera::rts_camera_system;
use bevy::render::render_graph::{RenderGraph, AssetRenderResourcesNode, base};
use bevy::render::pipeline::{PipelineDescriptor, PipelineSpecialization, DynamicBinding, RenderPipeline};
use bevy::render::shader::{ShaderStages, ShaderStage};

fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            vsync: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_asset::<MyMaterial>()
        .add_startup_system(setup.system())
        .add_system(rts_camera_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let generator = MapGenerator::new();
    let mesh_handles = generator.generate_meshes(&mut meshes);

    let mut translation = Vec3::new(0.0, 0.0, 0.0);

    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to our shader
    render_graph.add_system_node(
        "my_material",
        AssetRenderResourcesNode::<MyMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("my_material", base::node::MAIN_PASS)
        .unwrap();

    // Create a new material
    let material = materials.add(MyMaterial {
        color: Color::rgb(0.0, 0.8, 0.0),
    });

    for ((x, z), mesh) in mesh_handles {
        translation = Vec3::new(x as f32, 0.0, z as f32);
        commands
            .spawn(PbrComponents {
                mesh,
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
                    pipeline_handle,
                    // NOTE: in the future you wont need to manually declare dynamic bindings
                    PipelineSpecialization {
                        dynamic_bindings: vec![
                            // Transform
                            DynamicBinding {
                                bind_group: 1,
                                binding: 0,
                            },
                            // MapMaterial_color
                            DynamicBinding {
                                bind_group: 1,
                                binding: 1,
                            },
                        ],
                        ..Default::default()
                    },
                )]),
                transform: Transform::from_translation(translation),
                ..Default::default()
            })
            .with(material);
    }

    let italy = Vec3::new(552.0, 0.0, 377.26);
    let angle = std::f32::consts::PI / 4.0;
    let camera_state = rts_camera::State::new_looking_at_zoomed_out(italy, angle, 180.0);
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
