use bevy::render::pipeline::*;
use bevy::render::shader::{ShaderStages, ShaderStage};
use bevy::prelude::*;
use bevy::render::render_graph::*;
use bevy::render::renderer::RenderResources;
use once_cell::sync::OnceCell;

pub const VERTEX_SHADER: &str = include_str!("map.vert");
pub const FRAGMENT_SHADER: &str = include_str!("map.frag");

static PIPELINE: OnceCell<Handle<PipelineDescriptor>> = OnceCell::new();

#[derive(RenderResources, Default)]
pub struct MapMaterial {
    pub forest_texture: Handle<Texture>,
    pub beach_texture: Handle<Texture>,
}

#[derive(RenderResources, Default)]
pub struct TimeNode {
    time: f32,
}

pub fn setup(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    render_graph.add_system_node(
        "map_material",
        AssetRenderResourcesNode::<MapMaterial>::new(true),
    );

    render_graph
        .add_node_edge("map_material", base::node::MAIN_PASS)
        .unwrap();

    render_graph.add_system_node("time", RenderResourcesNode::<TimeNode>::new(true));

    PIPELINE.set(pipeline_handle).unwrap();
}

pub fn update_time(time: Res<Time>, mut nodes: Query<&mut TimeNode>) {
    for mut node in &mut nodes.iter() {
        node.time = time.seconds_since_startup as f32;
    }
}

pub fn render_pipelines() -> RenderPipelines {
    RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
        *PIPELINE.get().expect("map::shader::init must be called first!"),
        // NOTE: in the future you wont need to manually declare dynamic bindings
        PipelineSpecialization {
            dynamic_bindings: vec![
                // Transform
                DynamicBinding {
                    bind_group: 1,
                    binding: 0,
                },
                // MapMaterial_texture
                DynamicBinding {
                    bind_group: 1,
                    binding: 1,
                },
                // Time_time
                DynamicBinding {
                    bind_group: 2,
                    binding: 1,
                }
            ],
            ..Default::default()
        },
    )])
}
