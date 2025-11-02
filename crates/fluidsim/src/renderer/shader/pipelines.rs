use super::buffers::Buffers;

macro_rules! count {
    () => (0);
    ($el:ident $(, $rem:ident)*) => (1 + count!($($rem),*))
}

macro_rules! pipelines {
    (@dispatch particles, $p:ident) => ($p.div_ceil(::gpu_shared::WORKGROUP_SIZE));
    (@dispatch $n:expr, $p:ident) => ($n);

    (@x $p:ident) => (pipelines!(@dispatch particles, $p));
    (@y $p:ident) => (1);
    (@z $p:ident) => (1);

    (@x $p:ident $x:tt) => (pipelines!(@dispatch $x, $p));
    (@y $p:ident $y:tt) => (pipelines!(@dispatch $y, $p));
    (@z $p:ident $z:tt) => (pipelines!(@dispatch $z, $p));

    (@dispatcher $v:ident $i:expr;) => ({});

    (@dispatcher $v:ident $i:expr; pre_sort $($entries:ident)*) => {
        ::cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                $v.push((|s, e, _q, b, d, n| {
                    e.write_timestamp(&b.profiler.query_set, $i);
                    s.pre_sort.dispatch(e, d, n);
                    e.write_timestamp(&b.profiler.query_set, $i + 1);
                }) as Dispatcher);

                $v.push((|s, e, q, b, _d, n| {
                    e.write_timestamp(&b.profiler.query_set, $i + 2);
                    s.sorter.sort(e, q, &b.sort.sort_buffers, Some(n));
                    e.write_timestamp(&b.profiler.query_set, $i + 3);
                }) as Dispatcher);
            } else {
                $v.push((|s, e, _q, _b, d, n| {
                    s.pre_sort.dispatch(e, d, n);
                }) as Dispatcher);

                $v.push((|s, e, q, b, _d, n| {
                    s.sorter.sort(e, q, &b.sort.sort_buffers, Some(n));
                }) as Dispatcher);
            }
        }

        pipelines!(@dispatcher $v $i + 4; $($entries)*);
    };

    (@dispatcher $v:ident $i:expr; $entry:ident $($entries:ident)*) => {
        ::cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                $v.push((|s, e, _q, b, d, n| {
                    e.write_timestamp(&b.profiler.query_set, $i);
                    s.$entry.dispatch(e, d, n);
                    e.write_timestamp(&b.profiler.query_set, $i + 1);
                }) as Dispatcher);
            } else {
                $v.push((|s, e, _q, _b, d, n| {
                    s.$entry.dispatch(e, d, n);
                }) as Dispatcher);
            }
        }

        pipelines!(@dispatcher $v $i + 2; $($entries)*);
    };

    (@profile $perf:ident $ts:ident $per:ident $i:expr;) => {
        $perf.total = ($ts[$i - 1] - $ts[0]) as f32 * $per * 1e-6;
    };

    (@profile $perf:ident $ts:ident $per:ident $i:expr; pre_sort $($entries:ident)*) => {
        $perf.pre_sort = ($ts[$i + 1] - $ts[$i]) as f32 * $per * 1e-6;
        $perf.sort = ($ts[$i + 3] - $ts[$i + 2]) as f32 * $per * 1e-6;

        pipelines!(@profile $perf $ts $per $i + 4; $($entries)*);
    };

    (@profile $perf:ident $ts:ident $per:ident $i:expr; $entry:ident $($entries:ident)*) => {
        $perf.$entry = ($ts[$i + 1] - $ts[$i]) as f32 * $per * 1e-6;

        pipelines!(@profile $perf $ts $per $i + 2; $($entries)*);
    };

    ($(
        compute $entry:ident$([$x:tt; $y:tt; $z:tt])? as $cty:ident {
            $(from $group:ident use $($buffer:ident),+);+;
        }
    )+) => {
        pub const PIPELINES: usize = count!($($entry),+) + 1;

        type Dispatcher = for<'a> fn(
            &'a Pipelines,
            &'a mut wgpu::CommandEncoder,
            &'a wgpu::Queue,
            &'a super::buffers::Buffers,
            &'a wgpu::ComputePassDescriptor<'a>,
            u32,
        );

        $(
            pub struct $cty {
                bind_groups: [wgpu::BindGroup; Self::GROUPS],
                pipeline: wgpu::ComputePipeline,
            }

            impl $cty {
                pub const GROUPS: usize = count!($($group),+);

                pub fn new(device: &wgpu::Device, buffers: &Buffers, shader: &wgpu::ShaderModule) -> Self {
                    let bind_layouts = Self::bind_layouts(device, buffers);
                    let bind_groups = Self::bind_groups(device, &bind_layouts, buffers);
                    let pipeline_layout = Self::pipeline_layout(device, &bind_layouts);
                    let pipeline = Self::pipeline(device, &pipeline_layout, shader);

                    Self {
                        bind_groups,
                        pipeline,
                    }
                }

                fn bind_layouts(device: &wgpu::Device, buffers: &Buffers) -> [wgpu::BindGroupLayout; Self::GROUPS] {
                    let mut idx;

                    [$({
                        idx = 0;

                        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some(concat!("physics::", stringify!($entry), "$pipeline_layout::", stringify!($group), "$bgl")),
                            entries: &[$({
                                idx += 1;

                                wgpu::BindGroupLayoutEntry {
                                    binding: idx - 1,
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    ty: buffers.$group.$buffer.binding,
                                    count: None
                                }
                            }),+]
                        })
                    }),+]
                }


                fn bind_groups(
                    device: &wgpu::Device,
                    layouts: &[wgpu::BindGroupLayout],
                    buffers: &Buffers
                ) -> [wgpu::BindGroup; Self::GROUPS] {
                    let mut i = 0;

                    [$({
                        let mut j = 0;

                        let entries = [$({
                            j += 1;

                            wgpu::BindGroupEntry {
                                binding: j - 1,
                                resource: wgpu::BindingResource::Buffer(
                                    buffers.$group.$buffer.buffer.as_entire_buffer_binding()
                                )
                            }
                        }),+];

                        i += 1;

                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some(concat!("physics::", stringify!($entry), "$pipeline::", stringify!($group), "$bind_group")),
                            layout: &layouts[i - 1],
                            entries: &entries
                        })
                    }),+]
                }

                fn pipeline_layout(device: &wgpu::Device, layouts: &[wgpu::BindGroupLayout]) -> wgpu::PipelineLayout {
                    const _GROUPS: usize = $cty::GROUPS;

                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(concat!("physics::", stringify!($entry), "$pipeline_layout")),
                        bind_group_layouts: &::std::array::from_fn::<_, _GROUPS, _>(|i| &layouts[i]),
                        push_constant_ranges: &[]
                    })
                }

                fn pipeline(
                    device: &wgpu::Device,
                    layout: &wgpu::PipelineLayout,
                    shader: &wgpu::ShaderModule
                ) -> wgpu::ComputePipeline {
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(concat!("physics::", stringify!($entry), "$pipeline")),
                        layout: Some(layout),
                        module: shader,
                        entry_point: Some(stringify!($entry)),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    })
                }

                pub fn dispatch(
                    &self,
                    encoder: &mut wgpu::CommandEncoder,
                    descriptor: &wgpu::ComputePassDescriptor<'_>,
                    num_particles: u32,
                ) {
                    let mut pass = encoder.begin_compute_pass(descriptor);
                    pass.set_pipeline(&self.pipeline);
                    for (i, bg) in self.bind_groups.iter().enumerate() {
                        pass.set_bind_group(i as u32, bg, &[]);
                    }
                    pass.dispatch_workgroups(
                        pipelines!(@x num_particles $($x)?),
                        pipelines!(@y num_particles $($y)?),
                        pipelines!(@z num_particles $($z)?)
                    );
                }
            }
        )+

        #[derive(Default, Clone, Copy)]
        pub struct ComputeShaderPerformance {
            $(pub $entry: f32,)+
            pub sort: f32,
            pub total: f32,
        }

        impl ::std::fmt::Display for ComputeShaderPerformance {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "Compute Shader Performance (ms):\n")?;
                $(
                    write!(f, "  {:<20}:\t{}\n", stringify!($entry), self.$entry)?;
                )+
                write!(f, "  {:<20}:\t{}\n", "sort", self.sort)?;
                write!(f, "  {:<20}:\t{}\n", "total", self.total)
            }
        }

        pub struct Pipelines {
            $(pub $entry: $cty,)+
            pub sorter: ::wgpu_sort::GPUSorter,
        }

        impl Pipelines {
            pub fn new(
                device: &wgpu::Device,
                buffers: &Buffers,
                shader: &wgpu::ShaderModule,
                sorter: ::wgpu_sort::GPUSorter,
            ) -> Self {
                Self {
                    $($entry: $cty::new(device, buffers, shader),)+
                    sorter,
                }
            }

            #[allow(unused_assignments)]
            fn iter() -> impl Iterator<Item = Dispatcher> {
                let mut dispatchers = Vec::with_capacity(PIPELINES + 1);

                pipelines!(@dispatcher dispatchers 0; $($entry)*);

                dispatchers.into_iter()
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn dispatch_all(
                &self,
                device: &wgpu::Device,
                encoder: &mut wgpu::CommandEncoder,
                queue: &wgpu::Queue,
                buffers: &Buffers,
                descriptor: &wgpu::ComputePassDescriptor<'_>,
                num_particles: u32,
            ) -> wgpu::Buffer {
                for dispatch in Self::iter() {
                    dispatch(self, encoder, queue, buffers, descriptor, num_particles);
                }

                let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("profiler$readback_staging_buffer"),
                    size: buffers.profiler.query_buffer.size(),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                // copy to readback buffer
                encoder.resolve_query_set(
                    &buffers.profiler.query_set,
                    0..2 * PIPELINES as u32,
                    &buffers.profiler.query_buffer,
                    0,
                );
                encoder.copy_buffer_to_buffer(
                    &buffers.profiler.query_buffer,
                    0,
                    &readback_buffer,
                    0,
                    2 * PIPELINES as u64 * ::std::mem::size_of::<u64>() as u64,
                );

                readback_buffer
            }

            #[cfg(target_arch = "wasm32")]
            pub fn dispatch_all(
                &self,
                device: &wgpu::Device,
                encoder: &mut wgpu::CommandEncoder,
                queue: &wgpu::Queue,
                buffers: &Buffers,
                descriptor: &wgpu::ComputePassDescriptor<'_>,
                num_particles: u32,
            ) {
                for dispatch in Self::iter() {
                    dispatch(self, encoder, queue, buffers, descriptor, num_particles);
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn profile(&self, queue: &wgpu::Queue, readback_buffer: wgpu::Buffer, set_profiler: impl FnOnce(ComputeShaderPerformance) + Send + Sync + 'static) {
                let period = queue.get_timestamp_period(); // ns/tick

                readback_buffer.clone().slice(..).map_async(wgpu::MapMode::Read, move |_| {
                    let data = readback_buffer.slice(..).get_mapped_range();
                    let timestamps: &[u64] = bytemuck::cast_slice(&data);

                    let mut profiling = ComputeShaderPerformance {
                        $($entry: 0.0,)+
                        sort: 0.0,
                        total: 0.0,
                    };

                    pipelines!(@profile profiling timestamps period 0; $($entry)*);

                    set_profiler(profiling);
                });
            }
        }
    };
}

pipelines!(
    compute external_forces as ExternalForces {
        from user_data use settings, mouse;
        from physics use positions, predictions, velocities;
    }

    compute pre_sort as PreSort {
        from user_data use settings;
        from physics use predictions;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute post_sort as PostSort {
        from user_data use settings;
        from spatial_hash use indices;
        from sort use keys;
    }

    compute update_densities as UpdateDensities {
        from user_data use settings;
        from physics use predictions, densities;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute pressure_force as PressureForce {
        from user_data use settings;
        from physics use predictions, velocities, densities;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute viscosity as Viscosity {
        from user_data use settings;
        from physics use predictions, velocities;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute update_positions as UpdatePositions {
        from user_data use settings;
        from physics use positions, velocities;
    }

    compute collide as Collide {
        from user_data use settings;
        from physics use positions, velocities;
    }

    compute copy_prims as CopyPrims {
        from user_data use settings;
        from physics use positions, velocities;
        from drawing use primitives;
    }
);
