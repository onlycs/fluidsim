use glam::{UVec2, vec2};
use gpu_shared::SCALE;
use wgpu_sort::Sorter;

use super::pipelines::Pipelines;
use crate::{
    prelude::*,
    renderer::{buffers::Buffers, graphics::GraphicsContext, state::SimulationState},
};

static MAX_ARRAY: [u32; ARRAY_LEN] = [u32::MAX; ARRAY_LEN];
static EMPTY_VEC2: [[f32; 2]; ARRAY_LEN] = [[0.; 2]; ARRAY_LEN];

#[derive(Default)]
pub(crate) struct PhysicsUniformData {
    settings: SimSettings,
    mouse: MouseState,
}

impl PhysicsUniformData {
    pub(crate) fn num_particles(&self) -> u32 {
        self.settings.num_particles
    }

    pub(crate) fn particle_radius(&self) -> f32 {
        self.settings.particle_radius
    }
}

pub(crate) struct PhysicsShader {
    // uniform data
    pub(crate) udata: PhysicsUniformData,

    // all storage and uniform buffers
    buffers: Buffers,

    // state for updating the scene
    pub(crate) pipelines: Pipelines,
    pass_desc: wgpu::ComputePassDescriptor<'static>,
}

impl PhysicsShader {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let shader = device.create_shader_module(super::SHADER.clone());
        let sorter = Sorter::new(device, ARRAY_LEN as u32);

        let usr = PhysicsUniformData::default();
        let buffers = Buffers::new(device, &sorter);
        let pipelines = Pipelines::new(device, &buffers, &shader, sorter);

        // initialize some nonzero buffers
        buffers.spatial_hash.indices.reset(queue, &MAX_ARRAY);
        buffers.sort.lookup.reset(queue, &MAX_ARRAY);
        buffers.sort.keys.reset(queue, &MAX_ARRAY);

        let pass_descriptor = wgpu::ComputePassDescriptor {
            label: Some("physics/pass_descriptor"),
            timestamp_writes: None,
        };

        Self {
            udata: usr,
            buffers,
            pipelines,
            pass_desc: pass_descriptor,
        }
    }

    fn reset_resize(
        &mut self,
        rs: Option<UVec2>,
        ctx: &GraphicsContext,
        sim: &mut SimulationState,
    ) {
        let settings = &mut self.udata.settings;

        if let Some(size) = rs {
            settings.window_size = size;
        }

        let nx = sim.init.particles.x as f32;
        let ny = sim.init.particles.y as f32;
        let gap = sim.init.gap;
        let size = settings.particle_radius * 2.0;

        // calculate the position of the top-left particle
        let screen = settings.window_size.as_vec2() / SCALE;
        let half = screen / 2.;
        let ibox = vec2(nx * size + nx * gap - gap, ny * size + ny * gap - gap);
        let topleft = half - ibox / 2.;

        // calculate boundary conditions, 2d
        let sprest = (settings.mass / settings.target_density).sqrt();
        let r1_particles = ((screen + 2. * settings.particle_radius) / sprest).ceil(); // [x, y] the number of boundary particles in the first ring
        let r1_size = r1_particles * sprest; // [x, y] the size of the first ring of boundary particles
        let r1_diff = (r1_size - screen) / 2.; // [dx, dy] the difference between the screen and the first ring of boundary particles
        let r1_tl = -r1_diff; // [x, y] coordinates of the top-left particle in the first ring
        let r1_br = screen + r1_diff; // [x, y] coordinates of the bottom-right particle in the first ring

        // SAFETY: this would be zeroes anyways
        //
        // why? must create directly in the heap.
        // vec![...].into_boxed_slice() doesn't preserve length
        let mut positions = unsafe { Box::<[[f32; 2]; ARRAY_LEN]>::new_zeroed().assume_init() };

        let mut ctr = 0;
        for rn in 0..3 {
            let rn = rn as f32;

            let tl = r1_tl - rn * sprest;
            let br = r1_br + rn * sprest;
            let count = r1_particles + 2. * rn;

            for nx in 0..=count.x as usize {
                let nx = nx as f32;

                positions[ctr] = [tl.x + sprest * nx, tl.y];
                positions[ctr + 1] = [tl.x + sprest * nx, br.y];
                ctr += 2;
            }

            for ny in 1..count.y as usize {
                let ny = ny as f32;

                positions[ctr] = [tl.x, tl.y + sprest * ny];
                positions[ctr + 1] = [br.x, tl.y + sprest * ny];
                ctr += 2;
            }
        }

        settings.boundary_particles = ctr as u32;

        for i in 0..nx as usize {
            for j in 0..ny as usize {
                let offset = vec2(
                    size * i as f32 + gap * i as f32,
                    size * j as f32 + gap * j as f32,
                );

                // add a small random offset to the position because this engine is
                // deterministic
                let random = vec2(0.5 - rand::random::<f32>(), 0.5 - rand::random::<f32>());
                let pos = topleft + offset + random / 25.;

                positions[ctr] = [pos.x, pos.y];
                ctr += 1;
            }
        }

        settings.num_particles = ctr as u32;

        let queue = &ctx.queue;
        self.buffers.uniform.settings.reset(queue, &[*settings]);
        self.buffers.physics.positions.reset(queue, &positions);
        self.buffers.physics.predictions.reset(queue, &positions);
        self.buffers.physics.velocities.reset(queue, &EMPTY_VEC2);
        self.buffers.physics.densities.reset(queue, &EMPTY_VEC2);
        self.buffers.spatial_hash.indices.reset(queue, &MAX_ARRAY);
        self.buffers.sort.lookup.reset(queue, &MAX_ARRAY);
        self.buffers.sort.keys.reset(queue, &MAX_ARRAY);

        sim.time.pause();
    }

    pub(crate) fn reset(&mut self, ctx: &GraphicsContext, sim: &mut SimulationState) {
        self.reset_resize(None, ctx, sim);
    }

    pub(crate) fn resize(&mut self, size: UVec2, ctx: &GraphicsContext, sim: &mut SimulationState) {
        self.reset_resize(Some(size), ctx, sim);
    }

    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        dtime: f32,
    ) -> wgpu::Buffer {
        self.udata.settings.dtime = dtime;

        self.buffers
            .uniform
            .settings
            .reset(queue, &[self.udata.settings]);
        self.buffers.uniform.mouse.reset(queue, &[self.udata.mouse]);

        self.pipelines.dispatch_all(
            device,
            encoder,
            queue,
            &self.buffers,
            &self.pass_desc,
            self.udata.settings.num_particles,
        )
    }

    pub(crate) fn lease_panel(&mut self) -> &mut SimSettings {
        &mut self.udata.settings
    }

    pub(crate) fn set_mouse(&mut self, mouse: MouseState) {
        self.udata.mouse = mouse;
    }

    pub(crate) fn prims(&self) -> &wgpu::Buffer {
        &self.buffers.drawing.primitives.buffer
    }
}
