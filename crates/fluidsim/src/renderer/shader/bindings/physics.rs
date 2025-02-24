use std::mem;

use super::*;
use gpu_shared::ARRAY_LEN;

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
            usage: STORAGE_BUFFER,
            mapped_at_creation: false,
        }),
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("physics::simulation_data::predictions"),
            size: (mem::size_of::<[f32; 2]>() * ARRAY_LEN) as u64,
            usage: STORAGE_BUFFER,
            mapped_at_creation: false,
        }),
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("physics::simulation_data::velocities"),
            size: (mem::size_of::<[f32; 2]>() * ARRAY_LEN) as u64,
            usage: STORAGE_BUFFER,
            mapped_at_creation: false,
        }),
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("physics::simulation_data::densities"),
            size: (mem::size_of::<f32>() * ARRAY_LEN) as u64,
            usage: STORAGE_BUFFER,
            mapped_at_creation: false,
        }),
    ]
}
