use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_graph::{self},
        // render_resource::*,
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
    },
};

use crate::texture_a::*;
use crate::texture_c::*;
use crate::texture_d::*;

use std::borrow::Cow;

use crate::{
    CommonUniform, CommonUniformMeta, ShaderHandles, ShadertoyCanvas, ShadertoyPipelines,
    ShadertoyState, WORKGROUP_SIZE,
};

struct TextureBBindGroup {
    texture_b_bind_group: BindGroup,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

#[derive(Clone, Deref, ExtractResource)]
pub struct TextureB(pub Handle<Image>);

// pub struct TextureBPipeline {
//     texture_b_bind_group_layout: BindGroupLayout,
// }

// impl FromWorld for TextureBPipeline {
//     fn from_world(world: &mut World) -> Self {
//         let texture_b_bind_group_layout = world
//             .resource::<RenderDevice>()
//             .create_bind_group_layout(&BindGroupLayoutDescriptor {
//                 label: Some("layout_b"),
//                 entries: &[
//                     BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: ShaderStages::COMPUTE,
//                         ty: BindingType::Buffer {
//                             ty: BufferBindingType::Uniform,
//                             has_dynamic_offset: false,
//                             min_binding_size: BufferSize::new(
//                                 CommonUniform::std140_size_static() as u64
//                             ),
//                         },
//                         count: None,
//                     },
//                     BindGroupLayoutEntry {
//                         binding: 1,
//                         visibility: ShaderStages::COMPUTE,
//                         ty: BindingType::StorageTexture {
//                             access: StorageTextureAccess::ReadWrite,
//                             format: TextureFormat::Rgba32Float,
//                             view_dimension: TextureViewDimension::D2,
//                         },
//                         count: None,
//                     },
//                     BindGroupLayoutEntry {
//                         binding: 2,
//                         visibility: ShaderStages::COMPUTE,
//                         ty: BindingType::StorageTexture {
//                             access: StorageTextureAccess::ReadWrite,
//                             format: TextureFormat::Rgba32Float,
//                             view_dimension: TextureViewDimension::D2,
//                         },
//                         count: None,
//                     },
//                     BindGroupLayoutEntry {
//                         binding: 3,
//                         visibility: ShaderStages::COMPUTE,
//                         ty: BindingType::StorageTexture {
//                             access: StorageTextureAccess::ReadWrite,
//                             format: TextureFormat::Rgba32Float,
//                             view_dimension: TextureViewDimension::D2,
//                         },
//                         count: None,
//                     },
//                     BindGroupLayoutEntry {
//                         binding: 4,
//                         visibility: ShaderStages::COMPUTE,
//                         ty: BindingType::StorageTexture {
//                             access: StorageTextureAccess::ReadWrite,
//                             format: TextureFormat::Rgba32Float,
//                             view_dimension: TextureViewDimension::D2,
//                         },
//                         count: None,
//                     },
//                 ],
//             });

//         TextureBPipeline {
//             texture_b_bind_group_layout,
//         }
//     }
// }

// pub fn extract_texture_b(mut commands: Commands, image: Res<TextureB>) {
//     commands.insert_resource(TextureB(image.clone()));
// }

pub fn queue_bind_group_b(
    mut commands: Commands,
    pipeline: Res<ShadertoyPipelines>,
    gpu_images: Res<RenderAssets<Image>>,
    texture_a_image: Res<TextureA>,
    texture_b_image: Res<TextureB>,
    texture_c_image: Res<TextureC>,
    texture_d_image: Res<TextureD>,
    render_device: Res<RenderDevice>,
    mut pipeline_cache: ResMut<PipelineCache>,
    all_shader_handles: Res<ShaderHandles>,
    common_uniform_meta: ResMut<CommonUniformMeta>,
) {
    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: None,
        layout: Some(vec![pipeline.abcd_group_layout.clone()]),
        shader: all_shader_handles.texture_b_shader.clone(),
        shader_defs: vec!["INIT".to_string()],
        entry_point: Cow::from("update"),
    });

    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: None,
        layout: Some(vec![pipeline.abcd_group_layout.clone()]),
        shader: all_shader_handles.texture_b_shader.clone(),
        shader_defs: vec![],
        entry_point: Cow::from("update"),
    });

    let texture_a_view = &gpu_images[&texture_a_image.0];
    let texture_b_view = &gpu_images[&texture_b_image.0];
    let texture_c_view = &gpu_images[&texture_c_image.0];
    let texture_d_view = &gpu_images[&texture_d_image.0];

    let texture_b_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("binding b"),
        layout: &pipeline.abcd_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: common_uniform_meta.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture_a_view.texture_view),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&texture_b_view.texture_view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(&texture_c_view.texture_view),
            },
            BindGroupEntry {
                binding: 4,
                resource: BindingResource::TextureView(&texture_d_view.texture_view),
            },
        ],
    });

    commands.insert_resource(TextureBBindGroup {
        texture_b_bind_group,
        init_pipeline,
        update_pipeline,
    });
}

pub struct TextureBNode {
    pub state: ShadertoyState,
}

impl Default for TextureBNode {
    fn default() -> Self {
        Self {
            state: ShadertoyState::Loading,
        }
    }
}

impl render_graph::Node for TextureBNode {
    fn update(&mut self, world: &mut World) {
        let bind_group = world.resource::<TextureBBindGroup>();

        let pipeline_cache = world.resource::<PipelineCache>();

        let init_pipeline_cache = bind_group.init_pipeline;
        let update_pipeline_cache = bind_group.update_pipeline;

        match self.state {
            ShadertoyState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(init_pipeline_cache)
                {
                    self.state = ShadertoyState::Init
                }
            }
            ShadertoyState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(update_pipeline_cache)
                {
                    self.state = ShadertoyState::Update
                }
            }
            ShadertoyState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = world.resource::<TextureBBindGroup>();
        let canvas_size = world.resource::<ShadertoyCanvas>();
        let texture_b_bind_group = &bind_group.texture_b_bind_group;
        // let texture_a_bind_group = &bind_group.texture_a_bind_group;

        let init_pipeline_cache = bind_group.init_pipeline;
        let update_pipeline_cache = bind_group.update_pipeline;

        let pipeline_cache = world.resource::<PipelineCache>();

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        // pass.set_bind_group(0, texture_a_bind_group, &[]);
        pass.set_bind_group(0, texture_b_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            ShadertoyState::Loading => {}

            ShadertoyState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(init_pipeline_cache)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    canvas_size.width / WORKGROUP_SIZE,
                    canvas_size.height / WORKGROUP_SIZE,
                    1,
                );
            }

            ShadertoyState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(update_pipeline_cache)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(
                    canvas_size.width / WORKGROUP_SIZE,
                    canvas_size.height / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}
