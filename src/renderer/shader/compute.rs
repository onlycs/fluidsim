use crate::prelude::*;
use crate::renderer::WgpuData;
use core::f32;
use std::mem;
use std::ops::{Deref, DerefMut};

use super::vertex::CirclePrimitive;
use super::*;

pub struct PhysicsData {
    pub positions: Vec<[f32; 2]>,
    pub predictions: Vec<[f32; 2]>,
    pub velocities: Vec<[f32; 2]>,
    pub densities: Vec<f32>,
}

impl PhysicsData {
    pub fn bind_group_layout(
        device: &wgpu::Device,
        buffers: &[wgpu::Buffer; 4],
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physics::simulation_data$layout"),
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
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffers[1].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffers[2].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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

    pub fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffers: &[wgpu::Buffer; 4],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffers[0].as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(buffers[1].as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(buffers[2].as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(buffers[3].as_entire_buffer_binding()),
                },
            ],
            label: Some("physics::simulation_data$group"),
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
    pub fn bind_group_layout(
        device: &wgpu::Device,
        buffer: &[wgpu::Buffer; 2],
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physics::user_data$layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffer[0].size()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffer[1].size()),
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffers: &[wgpu::Buffer; 2],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffers[0].as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(buffers[1].as_entire_buffer_binding()),
                },
            ],
            label: Some("physics::user_data$group"),
        })
    }

    pub fn buffers(device: &wgpu::Device) -> [wgpu::Buffer; 2] {
        [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::user_data::settings"),
                size: mem::size_of::<RawSimSettings>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("physics::user_data::mouse"),
                size: mem::size_of::<RawMouseState>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ]
    }
}

pub struct SharedRenderingData {
    pub prims: Vec<CirclePrimitive>,
}

impl SharedRenderingData {
    pub fn bind_group_layout(
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physics::shared_data$layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(buffer.size()),
                },
                count: None,
            }],
        })
    }

    pub fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
            label: Some("physics::shared_data$group"),
        })
    }

    pub fn buffers(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("physics::shared_data::prims"),
            size: (mem::size_of::<CirclePrimitive>() * ARRAY_LEN) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

impl Default for SharedRenderingData {
    fn default() -> Self {
        Self {
            prims: vec![CirclePrimitive::default(); ARRAY_LEN],
        }
    }
}

pub struct Buffers {
    // group 0
    pub settings: wgpu::Buffer,
    pub mouse: wgpu::Buffer,

    // group 1
    pub positions: wgpu::Buffer,
    pub predictions: wgpu::Buffer,
    pub velocities: wgpu::Buffer,
    pub densities: wgpu::Buffer,

    // group 2
    pub prims: Arc<wgpu::Buffer>,
}

pub struct BindGroupLayouts {
    pub user: wgpu::BindGroupLayout,
    pub physics: wgpu::BindGroupLayout,
    pub rendering: wgpu::BindGroupLayout,
}

pub struct BindGroups {
    pub user: wgpu::BindGroup,
    pub physics: wgpu::BindGroup,
    pub rendering: wgpu::BindGroup,
}

pub struct UpdateState {
    pub mouse: bool,
    pub settings: bool,
    pub reset: bool,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            mouse: true,
            settings: true,
            reset: true,
        }
    }
}

pub struct ComputeData {
    pub physics: PhysicsData,
    pub user: UserData,
    pub shared: SharedRenderingData,

    pub buffers: Buffers,
    pub bind_layouts: BindGroupLayouts,
    pub bind_groups: BindGroups,
    pub update: UpdateState,
}

// creation and updating scene settings, etc
impl ComputeData {
    pub fn new(device: &wgpu::Device) -> Self {
        let user = UserData::default();
        let physics = PhysicsData::default();
        let shared = SharedRenderingData::default();

        let user_buffers = UserData::buffers(device);
        let physics_buffers = PhysicsData::buffers(device);
        let shared_buffer = SharedRenderingData::buffers(device);

        let user_bgl = UserData::bind_group_layout(device, &user_buffers);
        let physics_bgl = PhysicsData::bind_group_layout(device, &physics_buffers);
        let rendering_bgl = SharedRenderingData::bind_group_layout(device, &shared_buffer);

        let user_bg = UserData::bind_group(device, &user_bgl, &user_buffers);
        let physics_bg = PhysicsData::bind_group(device, &physics_bgl, &physics_buffers);
        let rendering_bg = SharedRenderingData::bind_group(device, &rendering_bgl, &shared_buffer);

        let [settings, mouse] = user_buffers;
        let [positions, predictions, velocities, densities] = physics_buffers;

        let buffers = Buffers {
            settings,
            mouse,
            positions,
            predictions,
            velocities,
            densities,
            prims: Arc::new(shared_buffer),
        };

        let bind_layouts = BindGroupLayouts {
            user: user_bgl,
            physics: physics_bgl,
            rendering: rendering_bgl,
        };

        let bind_groups = BindGroups {
            user: user_bg,
            physics: physics_bg,
            rendering: rendering_bg,
        };

        let this = Self {
            physics,
            user,
            shared,
            buffers,
            bind_layouts,
            bind_groups,
            update: Default::default(),
        };

        this
    }

    pub fn prims_buf(&self) -> Arc<wgpu::Buffer> {
        Arc::clone(&self.buffers.prims)
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
        self.user.settings.num_particles = nx as u32 * ny as u32;

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
    pub fn update(&mut self, queue: &wgpu::Queue, conditions: &InitialConditions) {
        if self.update.reset {
            self.reset(&conditions);

            queue.write_buffer(
                &self.buffers.positions,
                0,
                bytemuck::cast_slice(&self.physics.positions),
            );

            queue.write_buffer(
                &self.buffers.predictions,
                0,
                bytemuck::cast_slice(&self.physics.predictions),
            );

            queue.write_buffer(
                &self.buffers.velocities,
                0,
                bytemuck::cast_slice(&self.physics.velocities),
            );

            queue.write_buffer(
                &self.buffers.densities,
                0,
                bytemuck::cast_slice(&self.physics.densities),
            );

            // write so it's not empty. others will be written in first update()
            // call since we set all updates == true
            queue.write_buffer(
                &self.buffers.prims,
                0,
                bytemuck::cast_slice(&self.shared.prims),
            );

            self.update.reset = false;
        }

        if self.update.settings {
            queue.write_buffer(
                &self.buffers.settings,
                0,
                bytemuck::cast_slice(&[self.user.settings.into_raw()]),
            );

            self.update.settings = false;
        }

        if self.update.mouse {
            queue.write_buffer(
                &self.buffers.mouse,
                0,
                bytemuck::cast_slice(&[self.user.mouse.into_raw()]),
            );

            self.update.mouse = false;
        }
    }
}

#[derive(Default)]
pub struct ComputeState(Option<ComputeData>);

impl ComputeState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub fn init(&mut self, wgpu: &WgpuData) {
        self.0 = Some(ComputeData::new(&wgpu.device));
    }
}

impl Deref for ComputeState {
    type Target = ComputeData;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for ComputeState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
