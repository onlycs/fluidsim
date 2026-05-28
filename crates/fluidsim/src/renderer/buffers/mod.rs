use std::marker::PhantomData;

use bytemuck::NoUninit;
use gpu_shared::{ARRAY_LEN, MouseState};

use crate::{prelude::*, renderer::shader::circles::VsCirclePrimitive};

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
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data.as_ref()));
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
                                label: Some(concat!("physics/bindgroup:", stringify!($gid), "/buffer:", stringify!($bid))),
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
            pub keys: BufferBinding<[u32; ::gpu_shared::ARRAY_LEN]>,
            pub lookup: BufferBinding<[u32; ::gpu_shared::ARRAY_LEN]>,
        }

        impl Sort {
            pub fn new(keys: &wgpu::Buffer, values: &wgpu::Buffer) -> Self {
                Self {
                    lookup: BufferBinding {
                        buffer: Arc::new(keys.clone()),
                        binding: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(values.size()),
                        },
                        _phantom: ::std::marker::PhantomData,
                    },
                    keys: BufferBinding {
                        buffer: Arc::new(values.clone()),
                        binding: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(keys.size()),
                        },
                        _phantom: ::std::marker::PhantomData,
                    },
                }
            }
        }

        pub struct Buffers {
            $(pub $gid: $gty,)+
            pub sort: Sort,
        }

        impl Buffers {
            pub fn new(device: &wgpu::Device, sorter: &wgpu_sort::Sorter) -> Self {
                Self {
                    $($gid: $gty::buffers(&device),)+
                    sort: Sort::new(sorter.buffer_keys(), sorter.buffer_values()),
                }
            }
        }
    };
}

buffers!(
    group uniform(Uniform) {
        settings(SimSettings): uniform; COPY_DST,
        mouse(MouseState): uniform; COPY_DST,
    }

    group physics(Physics) {
        positions([[f32; 4]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST, // use vec4 for alignment/padding reasons, [x, y, z, w] where w is unused
        predictions([[f32; 4]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
        velocities([[f32; 4]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
        densities([[f32; 2]; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
    }

    group drawing(Drawing) {
        primitives([VsCirclePrimitive; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
    }

    group spatial_hash(SpatialHash) {
        indices([u32; ARRAY_LEN]): storage; COPY_SRC | COPY_DST,
    }
);
