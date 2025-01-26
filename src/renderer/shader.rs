use std::ops::{Deref, DerefMut};

use crate::{gradient::LinearGradient, prelude::*};
use bytemuck::{Pod, Zeroable};
use lyon::{
    geom::Point,
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor,
        StrokeVertex, StrokeVertexConstructor, TessellationError, VertexBuffers,
    },
};
use wgpu::{util::DeviceExt, BindGroupLayoutDescriptor, Color};

use super::WgpuData;

pub const PRIM_LEN: usize = 16384;

pub const FS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!("../shader/circle.fs.wgsl");
pub const VS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!("../shader/circle.vs.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VsCirclePrimitive {
    pub color: [f32; 4],
    pub translate: [f32; 2],
    pub z_index: i32,
    pub _pad: u32,
}

impl Default for VsCirclePrimitive {
    fn default() -> Self {
        Self {
            color: [0.; 4],
            translate: [0.; 2],
            z_index: 0,
            _pad: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VsGlobals {
    pub resolution: [f32; 2],
    pub scroll: [f32; 2],
    pub zoom: f32,
    pub _pad: [f32; 3],
}

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

#[allow(unused)]
pub struct VsData {
    pub globals: VsGlobals,
    pub prims: Vec<VsCirclePrimitive>,

    pub prims_buf: wgpu::Buffer,
    pub globals_buf: wgpu::Buffer,

    pub index_buf: wgpu::Buffer,
    pub vertex_buf: wgpu::Buffer,
    pub tesselation_buf: VertexBuffers<VsInput, u16>,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,

    pub fs: wgpu::ShaderModule,
    pub vs: wgpu::ShaderModule,

    pub retessellate: bool,
}

impl VsData {
    pub fn update(
        &mut self,
        wgpu: &WgpuData,
        settings: SimSettings,
        scene: &Scene,
    ) -> Result<(), DrawError> {
        if self.retessellate {
            let device = &wgpu.device;

            let mut tessellation_buf = VertexBuffers::new();
            let mut tessellator = FillTessellator::new();

            tessellator.tessellate_circle(
                Point::new(0., 0.),
                settings.radius * PX_PER_UNIT,
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
            self.tesselation_buf = tessellation_buf;
            self.retessellate = false;
        }

        let num_particles = scene.positions.len();
        let g = LinearGradient::new(vec![
            // #1747A2 rgb(23, 71, 162)
            (
                0.062,
                Color {
                    r: 23. / 255.,
                    g: 71. / 255.,
                    b: 162. / 255.,
                    a: 1.0,
                },
            ),
            // #51FC93 rgb(81, 252, 147)
            (
                0.48,
                Color {
                    r: 81. / 255.,
                    g: 252. / 255.,
                    b: 147. / 255.,
                    a: 1.0,
                },
            ),
            // #FCED06, rgb(252, 237, 6)
            (
                0.65,
                Color {
                    r: 252. / 255.,
                    g: 237. / 255.,
                    b: 6. / 255.,
                    a: 1.0,
                },
            ),
            // #EF4A0C, rgb(239, 74, 12)
            (
                1.0,
                Color {
                    r: 239. / 255.,
                    g: 74. / 255.,
                    b: 12. / 255.,
                    a: 1.0,
                },
            ),
        ]);

        // arbitrary max velocity because we need *color*
        // this isn't actually used for limiting the velocity
        const MAX_VEL: f32 = 15.0;

        // draw my particles
        for i in 0..num_particles {
            let Vec2 { x, y } = scene.positions[i] * PX_PER_UNIT;

            let speed = scene.velocities[i].distance(Vec2::ZERO);
            let relative = speed / MAX_VEL;
            let color = g.sample(relative.clamp(0.0, 1.0));

            self.prims[i] = VsCirclePrimitive {
                color: [
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    color.a as f32,
                ],
                translate: [x, y],
                z_index: 0,
                _pad: 0,
            };
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct VsState(Option<VsData>);

impl VsState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub fn init(&mut self, wgpu: &WgpuData) -> Result<(), TessellationError> {
        let mut tessellation_buf: VertexBuffers<_, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        tessellator.tessellate_circle(
            Point::new(0., 0.),
            100.,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut tessellation_buf, WithId(0)),
        )?;

        let device = &wgpu.device;

        let prims_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("primitive buffer"),
            size: (PRIM_LEN * std::mem::size_of::<VsCirclePrimitive>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals buffer"),
            size: std::mem::size_of::<VsGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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

        let fs = device.create_shader_module(FS);
        let vs = device.create_shader_module(VS);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("buffers layout"),
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
            label: Some("buffers bind group"),
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
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
            label: Some("circle pipeline layout"),
        });

        let fs_targets = [Some(wgpu::ColorTargetState {
            format: wgpu.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("circle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<VsInput>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &VsInput::descriptor(),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("main"),
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
            multiview: None,
            cache: None,
        };

        let pipeline = device.create_render_pipeline(&pipeline_desc);

        self.0 = Some(VsData {
            globals: VsGlobals {
                resolution: [0., 0.],
                scroll: [0., 0.],
                zoom: 1.0,
                _pad: [0.; _],
            },
            prims: vec![VsCirclePrimitive::default(); PRIM_LEN],
            prims_buf,
            globals_buf,
            index_buf: init_ibuf,
            vertex_buf: init_vbuf,
            tesselation_buf: tessellation_buf,
            bind_group_layout,
            bind_group,
            pipeline_layout,
            pipeline,
            fs,
            vs,
            retessellate: true,
        });

        Ok(())
    }
}

impl Deref for VsState {
    type Target = VsData;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for VsState {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
