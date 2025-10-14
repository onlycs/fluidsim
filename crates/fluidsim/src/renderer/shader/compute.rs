use std::ops::{Deref, DerefMut};
use wgpu_sort::GPUSorter;

use super::buffers::*;
use crate::prelude::*;
use crate::renderer::shader::pipelines::Pipelines;
use crate::renderer::wgpu_state::WgpuData;

const MAX_ARRAY: [u32; ARRAY_LEN] = [u32::MAX; ARRAY_LEN];

#[derive(Default)]
pub struct UserData {
    pub settings: SimSettings,
    pub mouse: MouseState,
}

pub struct UpdateState {
    pub mouse: bool,
    pub reset: bool,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            mouse: true,
            reset: true,
        }
    }
}

pub struct ComputeData {
    pub user: UserData,

    pub buffers: Buffers,
    pub update: UpdateState,

    pub pipelines: Pipelines,
    pub pass_desc: wgpu::ComputePassDescriptor<'static>,
}

// creation and updating scene settings, etc
impl ComputeData {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let shader = device.create_shader_module(super::SHADER.clone());

        let sorter = GPUSorter::new(device, 64);
        let len = NonZeroU32::new(ARRAY_LEN as u32).unwrap();
        let sort_buffers = sorter.create_sort_buffers(device, len);

        let usr = UserData::default();
        let sort = Sort::new(sort_buffers);
        let buffers = Buffers::new(device, sort);
        let pipelines = Pipelines::new(device, &buffers, &shader, sorter);

        // initialize some nonzero buffers
        buffers.spatial_hash.indices.reset(queue, &MAX_ARRAY);
        buffers.sort.lookup.reset(queue, &MAX_ARRAY);
        buffers.sort.keys.reset(queue, &MAX_ARRAY);

        let pass_descriptor = wgpu::ComputePassDescriptor {
            label: Some("physics$pass"),
            timestamp_writes: None,
        };

        Self {
            user: usr,
            buffers,
            update: Default::default(),
            pipelines,
            pass_desc: pass_descriptor,
        }
    }

    pub fn prims_buf(&self) -> Arc<wgpu::Buffer> {
        Arc::clone(&self.buffers.drawing.primitives.buffer)
    }

    pub fn reset(&mut self, conditions: &InitialConditions) -> Box<[[f32; 2]; ARRAY_LEN]> {
        let nx = conditions.particles.x;
        let ny = conditions.particles.y;
        let size = self.user.settings.particle_radius * 2.0;
        let gap = conditions.gap;

        // calculate the position of the top-left particle
        let topleft = -0.5
            * Vec2::new(
                (size * nx) + (gap * (nx - 1.)) - self.user.settings.particle_radius,
                (size * ny) + (gap * (ny - 1.)) - self.user.settings.particle_radius,
            );

        // clear all
        let mut positions = Box::new([[0.0; 2]; ARRAY_LEN]);
        self.user.settings.num_particles = nx as u32 * ny as u32;

        // create the particles
        let mut ctr = 0;
        for i in 0..nx as usize {
            for j in 0..ny as usize {
                let offset = Vec2::new(
                    size * i as f32 + gap * i as f32,
                    size * j as f32 + gap * j as f32,
                );

                // add a small random offset to the position because this engine is deterministic
                let random = Vec2::new(0.5 - rand::random::<f32>(), 0.5 - rand::random::<f32>());
                let pos = topleft + offset + random / 25.;

                positions[ctr] = [pos.x, pos.y];
                ctr += 1;
            }
        }

        positions
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        conditions: &InitialConditions,
        encoder: &mut wgpu::CommandEncoder,
        dtime: f32,
    ) -> Option<wgpu::Buffer> {
        self.user.settings.dtime = dtime;
        self.buffers
            .user_data
            .settings
            .reset(queue, &[self.user.settings]);

        if self.update.reset {
            let new_pos = self.reset(conditions);
            const EMPTY_VEC2: [[f32; 2]; ARRAY_LEN] = [[0f32; 2]; ARRAY_LEN];

            self.buffers.physics.positions.reset(queue, &new_pos);
            self.buffers.physics.predictions.reset(queue, &EMPTY_VEC2);
            self.buffers.physics.velocities.reset(queue, &EMPTY_VEC2);
            self.buffers.physics.densities.reset(queue, &EMPTY_VEC2);
            self.buffers.spatial_hash.indices.reset(queue, &MAX_ARRAY);
            self.buffers.sort.lookup.reset(queue, &MAX_ARRAY);
            self.buffers.sort.keys.reset(queue, &MAX_ARRAY);

            self.pipelines.copy_prims.dispatch(
                encoder,
                &self.pass_desc,
                self.user.settings.num_particles,
            );

            self.update.reset = false;
            return None;
        }

        if self.update.mouse {
            self.buffers
                .user_data
                .mouse
                .reset(queue, &[self.user.mouse]);

            self.update.mouse = false;
        }

        Some(self.pipelines.dispatch_all(
            device,
            encoder,
            queue,
            &self.buffers,
            &self.pass_desc,
            self.user.settings.num_particles,
        ))
    }
}

#[derive(Default)]
pub struct ComputeState(Option<ComputeData>);

impl ComputeState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub fn init(&mut self, wgpu: &WgpuData) {
        self.0 = Some(ComputeData::new(&wgpu.device, &wgpu.queue));
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
