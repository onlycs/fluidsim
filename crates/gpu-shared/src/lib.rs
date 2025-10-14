#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(not(target_arch = "spirv"))]
use glam::{Vec2, Vec4};

#[cfg(not(target_arch = "spirv"))]
use bytemuck::{Pod, Zeroable};

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct Settings {
    pub dtime: f32,

    pub gravity: f32,
    pub collision_damping: f32,

    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub mass: f32,

    pub interaction_radius: f32,
    pub interaction_strength: f32,

    pub viscosity_strength: f32,

    pub num_particles: u32,
    pub particle_radius: f32,

    pub window_size: Vec2,

    pub _pad1: u32,
    pub near_pressure_multiplier: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dtime: 0.002,

            gravity: 9.8,
            collision_damping: 0.40,

            smoothing_radius: 0.60,
            target_density: 20.0,

            near_pressure_multiplier: 18.0,
            pressure_multiplier: 500.0,
            viscosity_strength: 0.12,

            interaction_radius: 4.0,
            interaction_strength: 65.0,

            // window size
            #[cfg(not(target_arch = "wasm32"))]
            window_size: Vec2::new(1200., 800.),
            #[cfg(target_arch = "wasm32")]
            window_size: Vec2::new(1500., 1000.),

            #[cfg(not(target_arch = "wasm32"))]
            num_particles: 60 * 60,
            #[cfg(target_arch = "wasm32")]
            num_particles: 30 * 30,

            mass: 1.0,
            particle_radius: 0.045,

            _pad1: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct MouseState {
    pub position: Vec2,
    pub clickmask: u32,
    pub _pad: u32,
}

impl MouseState {
    pub fn new(px: Vec2, left: bool, right: bool) -> Self {
        Self {
            position: px,
            clickmask: (left as u32) | ((right as u32) << 1),
            ..Self::default()
        }
    }

    pub fn intensity(&self) -> f32 {
        if !self.active() {
            return 0.0;
        }

        if self.left() { 1.0 } else { -1.0 }
    }

    pub fn active(&self) -> bool {
        self.left() || self.right()
    }

    pub fn left(&self) -> bool {
        self.clickmask & 1 != 0
    }

    pub fn right(&self) -> bool {
        self.clickmask & 2 != 0
    }

    pub fn update(&mut self, px: Option<Vec2>, left: bool, right: bool) {
        self.position = px.unwrap_or(self.position);
        self.clickmask = (left as u32) | ((right as u32) << 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct Primitive {
    pub color: Vec4,
    pub translate: Vec2,
    pub z_index: i32,
    pub _pad: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct Globals {
    pub resolution: Vec2,
    pub scroll: Vec2,
    pub zoom: f32,
    pub _pad1: f32,
    pub _pad2: Vec2,
}

pub const SCALE: f32 = 100.0;
pub const ARRAY_LEN: usize = 16384;
pub const WORKGROUP_SIZE: u32 = 256;
