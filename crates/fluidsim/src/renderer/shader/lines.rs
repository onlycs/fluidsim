use std::mem;

use glam::{Quat, Vec3};
use gpu_shared::LineVertex;
use wgpu::util::DeviceExt;

pub struct LineShader {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    globals_bind: wgpu::BindGroup,
    vertex_count: u32,
}

fn axis_lines(len: f32) -> Vec<LineVertex> {
    let o = [0.0f32; 3];
    vec![
        LineVertex {
            position: o,
            color: [1.0, 0.2, 0.2],
        }, // X red
        LineVertex {
            position: [len, 0.0, 0.0],
            color: [1.0, 0.2, 0.2],
        },
        LineVertex {
            position: o,
            color: [0.2, 1.0, 0.2],
        }, // Y green
        LineVertex {
            position: [0.0, len, 0.0],
            color: [0.2, 1.0, 0.2],
        },
        LineVertex {
            position: o,
            color: [0.4, 0.4, 1.0],
        }, // Z blue
        LineVertex {
            position: [0.0, 0.0, len],
            color: [0.4, 0.4, 1.0],
        },
    ]
}

fn box_lines(size: Vec3, rot: Quat) -> Vec<LineVertex> {
    let c = [0.55f32, 0.55, 0.55];

    let corners = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(size.x, 0.0, 0.0),
        Vec3::new(size.x, size.y, 0.0),
        Vec3::new(0.0, size.y, 0.0),
        Vec3::new(0.0, 0.0, size.z),
        Vec3::new(size.x, 0.0, size.z),
        Vec3::new(size.x, size.y, size.z),
        Vec3::new(0.0, size.y, size.z),
    ];

    let corners: Vec<[f32; 3]> = corners.iter().map(|p| (rot * *p).to_array()).collect();

    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    edges
        .iter()
        .flat_map(|&(a, b)| {
            [
                LineVertex {
                    position: corners[a],
                    color: c,
                },
                LineVertex {
                    position: corners[b],
                    color: c,
                },
            ]
        })
        .collect()
}

impl LineShader {
    pub fn new(
        device: &wgpu::Device,
        surface_fmt: &wgpu::TextureFormat,
        globals_buf: &wgpu::Buffer,
        box_size: Vec3,
        box_quat: Quat,
    ) -> Self {
        // bind group layout: just globals
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lines/bindgroup_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lines/bindgroup"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });

        let shader = super::shader_module(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lines/pipeline_layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lines/pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_lines"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<LineVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3,  // position
                        1 => Float32x3,  // color
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_lines"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: *surface_fmt,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false), // lines don't write depth
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                ..Default::default()
            },
            cache: None,
            multiview_mask: None,
        });

        let vertices = Self::build_vertices(box_size, box_quat);
        let vertex_count = vertices.len() as u32;
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("lines/buffer:vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buf,
            globals_bind,
            vertex_count,
        }
    }

    fn build_vertices(box_size: Vec3, box_rot: Quat) -> Vec<LineVertex> {
        let mut v = Vec::new();
        v.extend(box_lines(box_size, box_rot));
        v.extend(axis_lines(box_size.x.min(box_size.y).min(box_size.z) * 0.5));
        v
    }

    pub fn rebuild(&mut self, device: &wgpu::Device, box_size: Vec3, box_rot: Quat) {
        let vertices = Self::build_vertices(box_size, box_rot);
        self.vertex_count = vertices.len() as u32;
        self.vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("lines/buffer:vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.globals_bind, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}
