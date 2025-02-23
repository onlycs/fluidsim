use std::ops::{Deref, DerefMut};
use wgpu_sort::GPUSorter;

use super::bindings::*;
use crate::prelude::*;
use crate::renderer::state::GameState;
use crate::renderer::wgpu_state::WgpuData;

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
    pub bind_groups: BindGroups,
    pub update: UpdateState,

    pub pipelines: [wgpu::ComputePipeline; 9],
    pub pass_desc: wgpu::ComputePassDescriptor<'static>,
    pub sorter: GPUSorter,
}

// creation and updating scene settings, etc
impl ComputeData {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let usr = UserData::default();
        let sorter = GPUSorter::new(device, 64);
        let len = NonZeroU32::new(ARRAY_LEN as u32).unwrap();
        let sort_bufs = sorter.create_sort_buffers(device, len);

        let user_buffers = user_data::buffers(device);
        let physics_buffers = physics::buffers(device);
        let shared = prims::buf(device);
        let starts = spatial_hash::starts_buf(device);
        let lookup = sort_bufs.values();
        let keys = sort_bufs.keys();

        let user_bgl = user_data::bind_group_layout(device, &user_buffers);
        let physics_bgl = physics::bind_group_layout(device, &physics_buffers);
        let rendering_bgl = prims::bind_group_layout(device, &shared);
        let spatial_bgl = spatial_hash::bind_group_layout(device, [lookup, &starts, keys]);

        let user_bg = user_data::bind_group(device, &user_bgl, &user_buffers);
        let physics_bg = physics::bind_group(device, &physics_bgl, &physics_buffers);
        let rendering_bg = prims::bind_group(device, &rendering_bgl, &shared);
        let spatial_bg = spatial_hash::bind_group(device, &spatial_bgl, [lookup, &starts, keys]);

        let [settings, mouse] = user_buffers;
        let [positions, predictions, velocities, densities] = physics_buffers;

        // initialize some nonzero buffers
        queue.write_buffer(lookup, 0, bytemuck::cast_slice(&vec![u32::MAX; ARRAY_LEN]));
        queue.write_buffer(&starts, 0, bytemuck::cast_slice(&vec![u32::MAX; ARRAY_LEN]));

        let buffers = Buffers {
            settings,
            mouse,
            positions,
            predictions,
            velocities,
            densities,
            prims: Arc::new(shared),
            starts,
            sort_bufs,
            debug: device.create_buffer(&Buffers::DEBUG_DESC),
        };

        let bind_layouts: BindGroupLayouts = BindGroupLayouts {
            user: user_bgl,
            physics: physics_bgl,
            rendering: rendering_bgl,
            spatial: spatial_bgl,
        };

        let bind_groups = BindGroups {
            user: user_bg,
            physics: physics_bg,
            rendering: rendering_bg,
            spatial: spatial_bg,
        };

        let shader = device.create_shader_module(super::SHADER.clone());

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("physics$layout"),
            bind_group_layouts: &[
                &bind_layouts.user,
                &bind_layouts.physics,
                &bind_layouts.rendering,
                &bind_layouts.spatial,
            ],
            push_constant_ranges: &[],
        });

        let pipeline_desc = [
            // 1. external forces
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::external_forces$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("external_forces"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // 2. update predictions
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::update_predictions$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("update_predictions"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // 3a. pre-sort
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::pre_sort$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("pre_sort"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // 3b. sort (not a pipeline)
            // 3c. post-sort
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::post_sort$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("post_sort"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // 4. update densities
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::update_densities$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("update_densities"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // 5. pressure forces
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::pressure_force$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("pressure_force"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // end: update positions
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::update_positions$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("update_positions"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // end: collision resolution
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::collide$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("collide"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
            // end: copy prims
            wgpu::ComputePipelineDescriptor {
                label: Some("physics::copy_prims$pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("copy_prims"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
        ];

        let pass = wgpu::ComputePassDescriptor {
            label: Some("physics$pass"),
            timestamp_writes: None,
        };

        let pipelines = pipeline_desc.map(|desc| device.create_compute_pipeline(&desc));

        Self {
            user: usr,
            buffers,
            bind_groups,
            update: Default::default(),
            pipelines,
            pass_desc: pass,
            sorter,
        }
    }

    pub fn prims_buf(&self) -> Arc<wgpu::Buffer> {
        Arc::clone(&self.buffers.prims)
    }

    pub fn reset(&mut self, conditions: &InitialConditions) -> Vec<[f32; 2]> {
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
        let mut positions = vec![[0.0; 2]; ARRAY_LEN];
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
        queue: &wgpu::Queue,
        conditions: &InitialConditions,
        encoder: &mut wgpu::CommandEncoder,
        dtime: f32,
    ) {
        let num_workgroups = self.user.settings.num_particles.div_ceil(WORKGROUP_SIZE);
        self.user.settings.dtime = dtime;
        queue.write_buffer(
            &self.buffers.settings,
            0,
            bytemuck::cast_slice(&[self.user.settings]),
        );

        if self.update.reset {
            let new_pos = self.reset(conditions);
            let empty_vec2s = vec![[0f32; 2]; ARRAY_LEN];
            let empty_f32s = vec![0f32; ARRAY_LEN];

            let Buffers {
                positions,
                predictions,
                velocities,
                densities,
                sort_bufs,
                starts,
                ..
            } = &self.buffers;

            let lookup = sort_bufs.values();
            queue.write_buffer(positions, 0, bytemuck::cast_slice(&new_pos));
            queue.write_buffer(predictions, 0, bytemuck::cast_slice(&empty_vec2s));
            queue.write_buffer(velocities, 0, bytemuck::cast_slice(&empty_vec2s));
            queue.write_buffer(densities, 0, bytemuck::cast_slice(&empty_f32s));
            queue.write_buffer(lookup, 0, bytemuck::cast_slice(&vec![u32::MAX; ARRAY_LEN]));
            queue.write_buffer(starts, 0, bytemuck::cast_slice(&vec![u32::MAX; ARRAY_LEN]));

            // run only the last pipeline on a reset
            let mut pass = encoder.begin_compute_pass(&self.pass_desc);
            pass.set_pipeline(&self.pipelines[self.pipelines.len() - 1]);
            pass.set_bind_group(0, &self.bind_groups.user, &[]);
            pass.set_bind_group(1, &self.bind_groups.physics, &[]);
            pass.set_bind_group(2, &self.bind_groups.rendering, &[]);
            pass.set_bind_group(3, &self.bind_groups.spatial, &[]);
            pass.dispatch_workgroups(num_workgroups, 1, 1);

            self.update.reset = false;
            return;
        }

        if self.update.mouse {
            queue.write_buffer(
                &self.buffers.mouse,
                0,
                bytemuck::cast_slice(&[self.user.mouse]),
            );

            self.update.mouse = false;
        }

        for (i, pipeline) in self.pipelines.iter().enumerate() {
            let mut pass = encoder.begin_compute_pass(&self.pass_desc);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.bind_groups.user, &[]);
            pass.set_bind_group(1, &self.bind_groups.physics, &[]);
            pass.set_bind_group(2, &self.bind_groups.rendering, &[]);
            pass.set_bind_group(3, &self.bind_groups.spatial, &[]);
            pass.dispatch_workgroups(num_workgroups, 1, 1);

            drop(pass);

            if i == 2 {
                self.sorter.sort(
                    encoder,
                    queue,
                    &self.buffers.sort_bufs,
                    Some(self.user.settings.num_particles),
                );
            }

            if i == 3 {
                // copy the sorted keys to the debug buffer
                encoder.copy_buffer_to_buffer(
                    &self.buffers.starts,
                    // &self.buffers.sort_bufs.keys(),
                    0,
                    &self.buffers.debug,
                    0,
                    1024,
                );
            }
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
