use crate::prelude::*;
use core::f32;
use std::mem;

use super::vertex::CirclePrimitive;
use super::*;

pub struct PhysicsData {
    pub positions: Vec<[f32; 2]>,
    pub predictions: Vec<[f32; 2]>,
    pub velocities: Vec<[f32; 2]>,
    pub densities: Vec<f32>,
}

impl PhysicsData {
    pub fn bind_group(device: &wgpu::Device, buffers: &[wgpu::Buffer; 4]) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physics::simulation_data"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffers[0].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffers[1].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffers[2].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffers[3].size()),
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn buffers(device: &wgpu::Device) -> [wgpu::Buffer; 4] {
        [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::simulation_data::positions"),
                size: (mem::size_of::<[f32; 2]>() * ARRAY_LEN) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::simulation_data::positions"),
                size: (mem::size_of::<[f32; 2]>() * ARRAY_LEN) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::simulation_data::positions"),
                size: (mem::size_of::<[f32; 2]>() * ARRAY_LEN) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::simulation_data::positions"),
                size: (mem::size_of::<f32>() * ARRAY_LEN) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ]
    }
}

impl Default for PhysicsData {
    fn default() -> Self {
        Self {
            positions: vec![[0.0; 2]; ARRAY_LEN],
            predictions: vec![[0.0; 2]; ARRAY_LEN],
            velocities: vec![[0.0; 2]; ARRAY_LEN],
            densities: vec![0.0; ARRAY_LEN],
        }
    }
}

#[derive(Default)]
pub struct UserData {
    pub settings: SimSettings,
    pub mouse: MouseState,
}

impl UserData {
    pub fn bind_group(device: &wgpu::Device, buffer: &[wgpu::Buffer; 2]) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physics::user_data"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffer[0].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffer[1].size()),
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn buffers(device: &wgpu::Device) -> [wgpu::Buffer; 2] {
        [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::user_data::settings"),
                size: mem::size_of::<RawSimSettings>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::user_data::mouse"),
                size: mem::size_of::<RawMouseState>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ]
    }
}

pub struct SharedRenderingData {
    pub prims: Vec<CirclePrimitive>,
}

impl Default for SharedRenderingData {
    fn default() -> Self {
        Self {
            prims: vec![CirclePrimitive::default(); ARRAY_LEN],
        }
    }
}

pub struct ComputeState {
    pub physics: PhysicsData,
    pub user: UserData,
    pub shared: SharedRenderingData,
}

// creation and updating scene settings, etc
impl ComputeState {
    pub fn new(conditions: &InitialConditions) -> Self {
        let mut this = Self {
            physics: Default::default(),
            user: Default::default(),
            shared: Default::default(),
        };

        this.reset(conditions);

        this
    }

    pub fn reset(&mut self, conditions: &InitialConditions) {
        let nx = conditions.particles.x as usize;
        let ny = conditions.particles.y as usize;
        let size = self.user.settings.particle_radius * 2.0;
        let gap = conditions.gap;

        // calculate the position of the top-left particle
        let topleft = -0.5
            * Vec2::new(
                (size * nx as f32) + (gap * (nx - 1) as f32) - self.user.settings.particle_radius,
                (size * ny as f32) + (gap * (ny - 1) as f32) - self.user.settings.particle_radius,
            );

        // clear all
        self.physics.positions.fill([0.0; 2]);
        self.physics.predictions.fill([0.0; 2]);
        self.physics.velocities.fill([0.0; 2]);
        self.physics.densities.fill(0.0);

        // create the particles
        let mut ctr = 0;
        for i in 0..nx {
            for j in 0..ny {
                let offset = Vec2::new(
                    size * i as f32 + gap * i as f32,
                    size * j as f32 + gap * j as f32,
                );

                // add a small random offset to the position because this engine is very deterministic
                let urandom = Vec2::new(
                    (0.5 - rand::random::<f32>()) / 10.,
                    (0.5 - rand::random::<f32>()) / 10.,
                );

                let pos = topleft + offset + urandom;

                self.physics.positions[ctr] = [pos.x, pos.y];
                ctr += 1;
            }
        }
    }

    // TODO: compute
    pub fn update(&mut self) {}
}
