use std::marker::PhantomData;

use crate::{prelude::*, renderer::shader::vertex::VsCirclePrimitive};
use bytemuck::NoUninit;
use gpu_shared::{ARRAY_LEN, MouseState};

pub struct BufferBinding<A> {
    pub buffer: Arc<wgpu::Buffer>,
    pub binding: wgpu::BindingType,
    _phantom: PhantomData<A>,
}

impl<A> BufferBinding<A> {
    pub fn reset<'a, K>(&'a self, queue: &wgpu::Queue, data: &'a A)
    where
        &'a A: AsRef<[K]>,
        K: NoUninit,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data.as_ref()))
    }
}

macro_rules! buffers {
    (@usage uniform) => (wgpu::BufferUsages::UNIFORM);
    (@usage storage) => (wgpu::BufferUsages::STORAGE);

    (@bbt uniform) => (wgpu::BufferBindingType::Uniform);
    (@bbt storage) => (wgpu::BufferBindingType::Storage { read_only: false });

    (@sliceof [$($x:tt)+]) => ([$($x)+]);
    (@sliceof $t:ident) => ([$t; 1]);

    ($(
        group $gid:ident($gty:ident) {$(
            $bid:ident($($bty:tt)+): $type:ident; $($usage:ident)|+
        ),+ $(,)?}
    )+) => {
        $(
            pub struct $gty {
                $(pub $bid: BufferBinding<buffers!(@sliceof $($bty)+)>),+
            }

            impl $gty {
                fn buffers(device: &wgpu::Device) -> Self {
                    Self {$(
                        $bid: BufferBinding {
                            buffer: Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some(concat!("physics::", stringify!($gid), "::", stringify!($bid), "$buffer")),
                                size: ::std::mem::size_of::<$($bty)+>() as u64,
                                usage: buffers!(@usage $type) | $(wgpu::BufferUsages::$usage)|+,
                                mapped_at_creation: false,
                            })),
                            binding: wgpu::BindingType::Buffer {
                                ty: buffers!(@bbt $type),
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(::std::mem::size_of::<$($bty)+>() as u64),
                            },
                            _phantom: ::std::marker::PhantomData,
                        }
                    ),+}
                }
            }
        )+

        pub struct Sort {
            pub lookup: BufferBinding<[u32; ::gpu_shared::ARRAY_LEN]>,
            pub keys: BufferBinding<[u32; ::gpu_shared::ARRAY_LEN]>,
            pub sort_buffers: ::wgpu_sort::SortBuffers,
        }

        impl Sort {
            pub fn new(buffers: ::wgpu_sort::SortBuffers) -> Self {
                let lookup = buffers.values();
                let keys = buffers.keys();

                Self {
                    lookup: BufferBinding {
                        buffer: Arc::new(lookup.clone()),
                        binding: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(lookup.size()),
                        },
                        _phantom: ::std::marker::PhantomData,
                    },
                    keys: BufferBinding {
                        buffer: Arc::new(keys.clone()),
                        binding: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(keys.size()),
                        },
                        _phantom: ::std::marker::PhantomData,
                    },
                    sort_buffers: buffers,
                }
            }
        }

        pub struct Profiler {
            pub query_set: wgpu::QuerySet,
            pub query_buffer: Arc<wgpu::Buffer>,
        }

        impl Profiler {
            fn new(device: &wgpu::Device) -> Self {
                let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("physics::profiler$query_set"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 2 * super::pipelines::PIPELINES as u32,
                });

                let query_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("physics::profiler$query_buffer"),
                    size: 2 * super::pipelines::PIPELINES as u64 * ::std::mem::size_of::<u64>() as u64,
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                    mapped_at_creation: false,
                }));

                Self {
                    query_set,
                    query_buffer,
                }
            }
        }

        pub struct Buffers {
            $(pub $gid: $gty,)+
            pub sort: Sort,
            pub profiler: Profiler,
        }

        impl Buffers {
            pub fn new(device: &wgpu::Device, sort: Sort) -> Self {
                Self {
                    $($gid: $gty::buffers(&device),)+
                    sort,
                    profiler: Profiler::new(device),
                }
            }
        }
    };
}

buffers!(
    group user_data(UserData) {
        settings(SimSettings): uniform; COPY_DST,
        mouse(MouseState): uniform; COPY_DST,
    }

    group physics(Physics) {
        positions([[f32; 2]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
        predictions([[f32; 2]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
        velocities([[f32; 2]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
        densities([[f32; 2]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
    }

    group drawing(Drawing) {
        primitives([VsCirclePrimitive; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
    }

    group spatial_hash(SpatialHash) {
        indices([u32; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
    }
);
