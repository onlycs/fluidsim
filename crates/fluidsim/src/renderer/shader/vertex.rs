use lyon::{
    geom::Point,
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor,
        StrokeVertex, StrokeVertexConstructor, TessellationError, VertexBuffers,
    },
};
use wgpu::{BindGroupLayoutDescriptor, util::DeviceExt};

use crate::{prelude::*, renderer::graphics::GraphicsContext};

pub type VsGlobals = gpu_shared::Globals;
pub type VsCirclePrimitive = gpu_shared::Primitive;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VsInput {
    pub position: [f32; 2],
    pub normal: [f32; 2],
    pub prim_id: u32,
}

impl VsInput {
    pub fn descriptor() -> [wgpu::VertexAttribute; 3] {
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32]
    }
}

pub struct WithId(pub u32);

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

pub struct VertexShaderContext {
    pub globals: VsGlobals,

    pub globals_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub vertex_buf: wgpu::Buffer,
    pub tessellation_buf: VertexBuffers<VsInput, u16>,

    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,

    pub retessellate: bool,
}

impl VertexShaderContext {
    #[allow(clippy::too_many_lines)]
    pub fn new(
        wgpu: &GraphicsContext,
        prims_buf: &wgpu::Buffer,
    ) -> Result<Self, TessellationError> {
        let mut tessellation_buf: VertexBuffers<_, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        tessellator.tessellate_circle(
            Point::new(0., 0.),
            100.,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut tessellation_buf, WithId(0)),
        )?;

        let device = &wgpu.device;

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vs::globals"),
            size: std::mem::size_of::<VsGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vs::index_buf"),
            contents: bytemuck::cast_slice(&tessellation_buf.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vs::vertex_buf"),
            contents: bytemuck::cast_slice(&tessellation_buf.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let fs = device.create_shader_module(super::SHADER.clone());
        let vs = device.create_shader_module(super::SHADER.clone());

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("physics/vertex/bindgroup_layout"),
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
            label: Some("physics/vertex/bindgroup"),
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
            label: Some("physics/vertex/pipeline_layout"),
            immediate_size: 0,
        });

        let fs_targets = [Some(wgpu::ColorTargetState {
            format: wgpu.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("physics/vertex/pipeline"),
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

        Ok(VertexShaderContext {
            globals: VsGlobals {
                resolution: Vec2::ZERO,
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
            retessellate: true,
        })
    }

    pub fn update(
        &mut self,
        wgpu: &GraphicsContext,
        settings: SimSettings,
    ) -> Result<(), DrawError> {
        if self.retessellate {
            let device = &wgpu.device;

            let mut tessellation_buf = VertexBuffers::new();
            let mut tessellator = FillTessellator::new();

            tessellator.tessellate_circle(
                Point::new(0., 0.),
                settings.particle_radius * PX_PER_UNIT,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut tessellation_buf, WithId(0)),
            )?;

            let init_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index buffer init"),
                contents: bytemuck::cast_slice(&tessellation_buf.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let init_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer init"),
                contents: bytemuck::cast_slice(&tessellation_buf.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            self.index_buf = init_ibuf;
            self.vertex_buf = init_vbuf;
            self.tessellation_buf = tessellation_buf;
            self.retessellate = false;
        }

        Ok(())
    }
}
