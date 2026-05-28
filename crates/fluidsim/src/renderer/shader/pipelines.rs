use crate::renderer::buffers::Buffers;

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
        $v.push((|s, _q, p, n| {
            s.pre_sort.dispatch(p, n);
        }) as Dispatcher);

        $v.push((|s, q, p, n| {
            s.sorter.sort_with_pass(p, q, n);
        }) as Dispatcher);

        pipelines!(@dispatcher $v $i + 4; $($entries)*);
    };

    (@dispatcher $v:ident $i:expr; $entry:ident $($entries:ident)*) => {
        $v.push((|s, _q, p, n| {
            s.$entry.dispatch(p, n);
        }) as Dispatcher);

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

        type Dispatcher = for<'a, 'b> fn(
            &'a Pipelines,
            &'a wgpu::Queue,
            &'a mut wgpu::ComputePass<'b>,
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
                            label: Some(concat!("physics/pipeline_layout:", stringify!($entry), "/bindgroup_layout:", stringify!($group))),
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
                            label: Some(concat!("physics/pipeline:", stringify!($entry), "/bindgroup:", stringify!($group))),
                            layout: &layouts[i - 1],
                            entries: &entries
                        })
                    }),+]
                }

                fn pipeline_layout(device: &wgpu::Device, layouts: &[wgpu::BindGroupLayout]) -> wgpu::PipelineLayout {
                    const _GROUPS: usize = $cty::GROUPS;

                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(concat!("physics/pipeline_layout:", stringify!($entry))),
                        bind_group_layouts: &::std::array::from_fn::<_, _GROUPS, _>(|i| Some(&layouts[i])),
                        immediate_size: 0
                    })
                }

                fn pipeline(
                    device: &wgpu::Device,
                    layout: &wgpu::PipelineLayout,
                    shader: &wgpu::ShaderModule
                ) -> wgpu::ComputePipeline {
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(concat!("physics/pipeline:", stringify!($entry))),
                        layout: Some(layout),
                        module: shader,
                        entry_point: Some(stringify!($entry)),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    })
                }

                pub fn dispatch(
                    &self,
                    pass: &mut wgpu::ComputePass,
                    num_particles: u32,
                ) {
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

        pub struct Pipelines {
            $(pub $entry: $cty,)+
            pub sorter: ::wgpu_sort::Sorter,
        }

        impl Pipelines {
            pub fn new(
                device: &wgpu::Device,
                buffers: &Buffers,
                shader: &wgpu::ShaderModule,
                sorter: ::wgpu_sort::Sorter,
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

            pub fn dispatch_all(
                &self,
                encoder: &mut wgpu::CommandEncoder,
                queue: &wgpu::Queue,
                descriptor: &wgpu::ComputePassDescriptor<'_>,
                num_particles: u32,
            ) {
                {
                    let mut pass = encoder.begin_compute_pass(descriptor);

                    for dispatch in Self::iter() {
                        dispatch(self, queue, &mut pass, num_particles);
                    }
                }
            }
        }
    };
}

pipelines!(
    compute external_forces as ExternalForces {
        from uniform use settings, mouse;
        from physics use positions, predictions, velocities;
    }

    compute pre_sort as PreSort {
        from uniform use settings;
        from physics use predictions;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute post_sort as PostSort {
        from uniform use settings;
        from spatial_hash use indices;
        from sort use keys;
    }

    compute update_densities as UpdateDensities {
        from uniform use settings;
        from physics use predictions, densities;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute pressure_force as PressureForce {
        from uniform use settings;
        from physics use predictions, velocities, densities;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute viscosity as Viscosity {
        from uniform use settings;
        from physics use predictions, velocities;
        from spatial_hash use indices;
        from sort use lookup, keys;
    }

    compute update_positions as UpdatePositions {
        from uniform use settings;
        from physics use positions, velocities;
    }

    compute collide as Collide {
        from uniform use settings;
        from physics use positions, velocities;
    }

    compute copy_prims as CopyPrims {
        from uniform use settings;
        from physics use positions, velocities;
        from drawing use primitives;
    }
);
