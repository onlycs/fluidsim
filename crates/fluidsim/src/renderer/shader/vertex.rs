use gpu_shared::SCALE;
use lyon::{
    geom::Point,
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor,
        StrokeVertex, StrokeVertexConstructor, TessellationError, VertexBuffers,
    },
};
use wgpu::{BindGroupLayoutDescriptor, util::DeviceExt};

use crate::{
    prelude::*,
    renderer::{graphics::GraphicsContext, shader::compute::PhysicsUniformData},
};

pub(crate) type VsGlobals = gpu_shared::Globals;
pub(crate) type VsCirclePrimitive = gpu_shared::Primitive;

#[derive(Debug, Snafu)]
pub(crate) enum VertexShaderError {
    #[snafu(display("At {location}: tessellation error\n{source}"))]
    Tessellation {
        source: TessellationError,
        #[snafu(implicit)]
        location: Location,
    },
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct VsInput {
    pub(crate) position: [f32; 2],
    pub(crate) normal: [f32; 2],
    pub(crate) prim_id: u32,
}

impl VsInput {
    pub(crate) fn descriptor() -> [wgpu::VertexAttribute; 3] {
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32]
    }
}

pub(crate) struct WithId(pub(crate) u32);

impl FillVertexConstructor<VsInput> for WithId {
    fn new_vertex(&mut self, vertex: FillVertex) -> VsInput {
        VsInput {
            position: vertex.position().to_array(),
            normal: [0.; 2],
            prim_id: self.0,
        }
    }
}

impl StrokeVertexConstructor<VsInput> for WithId {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> VsInput {
        VsInput {
            position: vertex.position().to_array(),
            normal: vertex.normal().to_array(),
            prim_id: self.0,
        }
    }
}

pub(crate) struct CircleShader {
    globals: VsGlobals,

    globals_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    vertex_buf: wgpu::Buffer,
    tessellation_buf: VertexBuffers<VsInput, u16>,

    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    msaa_tex: wgpu::Texture,
    msaa_view: wgpu::TextureView,
}

impl CircleShader {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn new(
        wgpu: &GraphicsContext,
        prims_buf: &wgpu::Buffer,
        screen: UVec2,
        udata: &PhysicsUniformData,
    ) -> Result<Self, VertexShaderError> {
        let mut tessellation_buf: VertexBuffers<_, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        tessellator
            .tessellate_circle(
                Point::new(0., 0.),
                udata.particle_radius() * SCALE,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut tessellation_buf, WithId(0)),
            )
            .context(TessellationSnafu)?;

        let device = &wgpu.device;

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("circle/buffer:globals"),
            size: std::mem::size_of::<VsGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("circle/buffer:index"),
            contents: bytemuck::cast_slice(&tessellation_buf.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("circle/buffer:vertex"),
            contents: bytemuck::cast_slice(&tessellation_buf.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let fs = device.create_shader_module(super::SHADER.clone());
        let vs = device.create_shader_module(super::SHADER.clone());

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("circle/bindgroup_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_buf.size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(prims_buf.size()),
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
                    resource: wgpu::BindingResource::Buffer(globals_buf.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(prims_buf.as_entire_buffer_binding()),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[Some(&bind_group_layout)],
            label: Some("circle/pipeline_layout"),
            immediate_size: 0,
        });

        let fs_targets = [Some(wgpu::ColorTargetState {
            format: wgpu.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("circle/pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<VsInput>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &VsInput::descriptor(),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("fs_main"),
                targets: &fs_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        };

        let pipeline = device.create_render_pipeline(&pipeline_desc);

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

        Ok(CircleShader {
            globals: VsGlobals {
                resolution: UVec2::ZERO,
                scroll: Vec2::ZERO,
                zoom: 1.0,
                ..VsGlobals::default()
            },
            globals_buf,
            index_buf,
            vertex_buf,
            tessellation_buf,
            bind_group,
            pipeline,
            msaa_tex,
            msaa_view,
        })
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
    }

    pub(crate) fn retesselate(
        &mut self,
        ctx: &GraphicsContext,
        radius: f32,
    ) -> Result<(), VertexShaderError> {
        let mut tessellation_buf = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        tessellator
            .tessellate_circle(
                Point::new(0., 0.),
                radius * SCALE,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut tessellation_buf, WithId(0)),
            )
            .context(TessellationSnafu)?;

        let index_buf = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("circle/buffer:index"),
                contents: bytemuck::cast_slice(&tessellation_buf.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let vertex_buf = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("circle/buffer:vertex"),
                contents: bytemuck::cast_slice(&tessellation_buf.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        self.index_buf = index_buf;
        self.vertex_buf = vertex_buf;
        self.tessellation_buf = tessellation_buf;

        Ok(())
    }

    pub(crate) fn draw(
        &self,
        ctx: &GraphicsContext,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        udata: &PhysicsUniformData,
    ) {
        ctx.queue
            .write_buffer(&self.globals_buf, 0, bytemuck::cast_slice(&[self.globals]));

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_view,
                    resolve_target: Some(surface_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            pass.draw_indexed(
                0..self.index_buf.size() as u32 / 2,
                0,
                0..udata.num_particles(),
            );
        }
    }

    pub(crate) fn lease_zoom(&mut self) -> &mut f32 {
        &mut self.globals.zoom
    }
}
