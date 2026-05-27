use wgpu::{BindGroupLayoutDescriptor, util::DeviceExt};

use crate::{
    prelude::*,
    renderer::{
        buffers::Buffers,
        graphics::GraphicsContext,
        shader::{lines::LineShader, physics::PhysicsUniformData},
        state::SimulationState,
    },
};

pub(crate) type VsGlobals = gpu_shared::Globals;
pub(crate) type VsCirclePrimitive = gpu_shared::Primitive;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct VsInput {
    pub(crate) position: [f32; 2],
    pub(crate) prim_id: u32,
}

impl VsInput {
    pub(crate) fn descriptor() -> [wgpu::VertexAttribute; 2] {
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32]
    }
}

pub(crate) struct CircleShader {
    globals: VsGlobals,

    globals_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    vertex_buf: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    msaa_tex: wgpu::Texture,
    msaa_view: wgpu::TextureView,
    depth_tex: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl CircleShader {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn new(wgpu: &GraphicsContext, buffers: &Buffers, screen: UVec2) -> Self {
        let device = &wgpu.device;

        let vertices = [
            VsInput {
                position: [-1., -1.],
                prim_id: 0,
            },
            VsInput {
                position: [1., -1.],
                prim_id: 0,
            },
            VsInput {
                position: [1., 1.],
                prim_id: 0,
            },
            VsInput {
                position: [-1., 1.],
                prim_id: 0,
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("circle/buffer:vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("circle/buffer:index"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("circle/buffer:globals"),
            size: std::mem::size_of::<VsGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = super::shader_module(device);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("circle/bindgroup_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // globals needed in fs_main for projection matrix (depth calculation)
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_buf.size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: buffers.uniform.settings.binding,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            buffers.drawing.primitives.buffer.size(),
                        ),
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("circle/bindgroup"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.uniform.settings.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.drawing.primitives.buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("circle/pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let (depth_tex, depth_view) = Self::create_depth(device, screen);

        let msaa_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("circle/texture:msaa"),
            size: wgpu::Extent3d {
                width: screen.x,
                height: screen.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let msaa_view = msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("circle/pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<VsInput>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &VsInput::descriptor(),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            globals: VsGlobals::default(),
            globals_buf,
            index_buf,
            vertex_buf,
            bind_group,
            pipeline,
            msaa_tex,
            msaa_view,
            depth_tex,
            depth_view,
        }
    }

    fn create_depth(device: &wgpu::Device, screen: UVec2) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("circle/texture:depth"),
            size: wgpu::Extent3d {
                width: screen.x,
                height: screen.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4, // must match MSAA count
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    pub(crate) fn resize(&mut self, ctx: &GraphicsContext, screen: UVec2) {
        self.globals.resolution = screen;

        self.msaa_tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("circle/texture:msaa"),
            size: wgpu::Extent3d {
                width: screen.x,
                height: screen.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: self.msaa_tex.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.msaa_view = self
            .msaa_tex
            .create_view(&wgpu::TextureViewDescriptor::default());

        (self.depth_tex, self.depth_view) = Self::create_depth(&ctx.device, screen);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &mut self,
        ctx: &GraphicsContext,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        state: &SimulationState,
        udata: &PhysicsUniformData,
        lines: &LineShader,
        screen: UVec2,
    ) {
        self.globals.view = state.player.view_matrix();
        self.globals.projection = state.player.projection_matrix(screen);

        ctx.queue
            .write_buffer(&self.globals_buf, 0, bytemuck::cast_slice(&[self.globals]));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.msaa_view,
                resolve_target: Some(surface_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.draw_indexed(
            0..6,
            0,
            udata.boundary_particles()..udata.num_particles(), // skip boundary
        );

        lines.draw(&mut pass);
    }

    pub(crate) fn globals_buf(&self) -> &wgpu::Buffer {
        &self.globals_buf
    }
}
