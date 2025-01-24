use std::ops::{Deref, DerefMut};

use crate::prelude::*;
use bytemuck::{Pod, Zeroable};
use lyon::{
    geom::Point,
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor,
        StrokeVertex, StrokeVertexConstructor, TessellationError, VertexBuffers,
    },
};
use wgpu::{util::DeviceExt, BindGroupLayoutDescriptor};

use super::WgpuState;

pub const PRIM_LEN: usize = 256;

pub const FS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!("../shader/circle.fs.wgsl");
pub const VS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!("../shader/circle.vs.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VsCirclePrimitive {
    pub color: [f32; 4],
    pub translate: [f32; 2],
    pub z_index: i32,
    pub _pad: i32,
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
    pub _pad: f32,
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
}

impl VsData {
    pub fn update(&mut self) -> Result<(), DrawError> {
        // lyon: tesselate a circle
        let mut buf = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        tessellator.tessellate_circle(
            Point::new(0., 0.),
            100.,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut buf, WithId(0)),
        )?;

        self.prims = Vec::with_capacity(256);

        self.prims.push(VsCirclePrimitive {
            color: [1., 0., 0., 1.],
            translate: [0., 0.],
            z_index: 0,
            _pad: 0,
        });
        self.prims.extend(vec![
            VsCirclePrimitive {
                color: [0.; 4],
                translate: [0.; 2],
                z_index: 0,
                _pad: 0,
            };
            PRIM_LEN - 2
        ]);

        self.tesselation_buf = buf;

        Ok(())
    }
}

#[derive(Default)]
pub struct VsState(pub Option<VsData>);

impl VsState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub fn create(&mut self, wgpu: &WgpuState) -> Result<(), TessellationError> {
        let mut tesselation_buf: VertexBuffers<_, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        tessellator.tessellate_circle(
            Point::new(0., 0.),
            100.,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut tesselation_buf, WithId(0)),
        )?;

        let device = &wgpu.device;

        let prims_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("primitive buffer"),
            size: (PRIM_LEN * std::mem::size_of::<VsCirclePrimitive>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals buffer"),
            size: std::mem::size_of::<VsGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        println!(
            "{} {}",
            std::mem::size_of::<VsGlobals>() as u64,
            globals_buf.size()
        );

        let init_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("init buffer"),
            contents: bytemuck::cast_slice(&tesselation_buf.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let init_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("init buffer"),
            contents: bytemuck::cast_slice(&tesselation_buf.vertices),
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
                        ty: wgpu::BufferBindingType::Uniform,
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
                zoom: 1.,
                _pad: 0.,
            },
            prims: Vec::new(),
            prims_buf,
            globals_buf,
            index_buf: init_ibuf,
            vertex_buf: init_vbuf,
            tesselation_buf,
            bind_group_layout,
            bind_group,
            pipeline_layout,
            pipeline,
            fs,
            vs,
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
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
