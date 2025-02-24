use std::sync::Arc;

use wgpu::BufferUsages;
use wgpu_sort::SortBuffers;

pub mod physics;
pub mod prims;
pub mod spatial_hash;
pub mod user_data;

const STORAGE_BUFFER: wgpu::BufferUsages = BufferUsages::STORAGE
    .union(BufferUsages::COPY_SRC)
    .union(BufferUsages::COPY_DST);

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

    // group 3
    pub sort_bufs: SortBuffers, // lookup=0, keys=2
    pub starts: wgpu::Buffer,   // starts=1
}

pub struct BindGroupLayouts {
    pub user: wgpu::BindGroupLayout,
    pub physics: wgpu::BindGroupLayout,
    pub rendering: wgpu::BindGroupLayout,
    pub spatial: wgpu::BindGroupLayout,
}

pub struct BindGroups {
    pub user: wgpu::BindGroup,
    pub physics: wgpu::BindGroup,
    pub rendering: wgpu::BindGroup,
    pub spatial: wgpu::BindGroup,
}
