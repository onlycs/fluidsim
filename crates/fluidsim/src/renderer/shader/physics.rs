use glam::vec3;
use wgpu_sort::Sorter;

use super::pipelines::Pipelines;
use crate::{
    prelude::*,
    renderer::{buffers::Buffers, graphics::GraphicsContext, state::SimulationState},
};

static MAX_ARRAY: [u32; ARRAY_LEN] = [u32::MAX; ARRAY_LEN];
static EMPTY_VEC2: [[f32; 2]; ARRAY_LEN] = [[0.; 2]; ARRAY_LEN];
static EMPTY_VEC4: [[f32; 4]; ARRAY_LEN] = [[0.; 4]; ARRAY_LEN];

#[derive(Default)]
pub(crate) struct PhysicsUniformData {
    settings: SimSettings,
    mouse: MouseState,
}

impl PhysicsUniformData {
    pub(crate) fn num_particles(&self) -> u32 {
        self.settings.num_particles
    }

    pub(crate) fn boundary_particles(&self) -> u32 {
        self.settings.boundary_particles
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
        let shader = super::shader_module(device);
        let sorter = Sorter::new(device, ARRAY_LEN as u32);

        let usr = PhysicsUniformData::default();
        let buffers = Buffers::new(device, &sorter);
        let pipelines = Pipelines::new(device, &buffers, shader, sorter);

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

    pub(crate) fn reset(&mut self, ctx: &GraphicsContext, sim: &mut SimulationState) {
        let settings = &mut self.udata.settings;
        settings.box_size = sim.init.box_size;
        settings.box_quat = sim.init.box_quat;

        let nx = sim.init.particles.x;
        let ny = sim.init.particles.y;
        let nz = sim.init.particles.z;
        let gap = sim.init.gap;
        let size = settings.particle_radius * 2.0;
        let box_size = settings.box_size;
        let rot = sim.init.box_quat;

        // calculate the position of the top-left particle
        let screen = settings.box_size;
        let half = screen / 2.;
        let ibox = (sim.init.particles.as_vec3() * (size + gap)) - gap;
        let topleft = half - ibox / 2.;

        // calculate boundary conditions, 3d (6 faces + 12 edges + 8 corners per shell)
        let sprest = (settings.mass / settings.target_density).cbrt();
        let r1_count = ((box_size + 2. * settings.particle_radius) / sprest).ceil();
        let r1_size_vec = r1_count * sprest;
        let r1_diff = (r1_size_vec - box_size) / 2.;
        let r1_tl = -r1_diff;

        // SAFETY: this would be zeroes anyways. must create directly on the heap;
        // vec![...].into_boxed_slice() doesn't preserve length.
        let mut positions = unsafe { Box::<[[f32; 4]; ARRAY_LEN]>::new_zeroed().assume_init() };

        let mut ctr = 0;
        for rn in 0..2 {
            let rn = rn as f32;
            let tl = r1_tl - rn * sprest;
            let count = (r1_count + 2. * rn).as_uvec3();

            let cx = count.x;
            let cy = count.y;
            let cz = count.z;

            // walk the (cx+1) × (cy+1) × (cz+1) grid; keep only positions on the hull
            for i in 0..=cx {
                for j in 0..=cy {
                    for k in 0..=cz {
                        let on_hull = i == 0 || i == cx || j == 0 || j == cy || k == 0 || k == cz;
                        if !on_hull {
                            continue;
                        }

                        let p = vec3(
                            tl.x + sprest * i as f32,
                            tl.y + sprest * j as f32,
                            tl.z + sprest * k as f32,
                        );
                        let p = rot * p;

                        positions[ctr] = [p.x, p.y, p.z, 0.]; // padding for alignment
                        ctr += 1;
                    }
                }
            }
        }

        settings.boundary_particles = ctr as u32;

        for i in 0..nx as usize {
            for j in 0..ny as usize {
                for k in 0..nz as usize {
                    let offset = vec3(
                        size * i as f32 + gap * i as f32,
                        size * j as f32 + gap * j as f32,
                        size * k as f32 + gap * k as f32,
                    );

                    // add a small random offset to the position because this engine is
                    // deterministic
                    let random = vec3(
                        rand::random::<f32>() - 0.5,
                        rand::random::<f32>() - 0.5,
                        rand::random::<f32>() - 0.5,
                    );
                    let pos = topleft + offset + random / 25.;
                    let pos = rot * pos;

                    positions[ctr] = [pos.x, pos.y, pos.z, 0.]; // padding for alignment
                    ctr += 1;
                }
            }
        }

        settings.num_particles = ctr as u32;

        let queue = &ctx.queue;
        self.buffers.uniform.settings.reset(queue, &[*settings]);
        self.buffers.physics.positions.reset(queue, &positions);
        self.buffers.physics.predictions.reset(queue, &positions);
        self.buffers.physics.velocities.reset(queue, &EMPTY_VEC4);
        self.buffers.physics.densities.reset(queue, &EMPTY_VEC2);
        self.buffers.spatial_hash.indices.reset(queue, &MAX_ARRAY);
        self.buffers.sort.lookup.reset(queue, &MAX_ARRAY);
        self.buffers.sort.keys.reset(queue, &MAX_ARRAY);

        sim.time.pause();
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

    pub(crate) fn buffers(&self) -> &Buffers {
        &self.buffers
    }
}
