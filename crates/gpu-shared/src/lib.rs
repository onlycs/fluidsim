#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(not(target_arch = "spirv"))]
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, UVec2, UVec3, Vec2, Vec3, Vec4, vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::glam;

pub const DEFAULT_BOX_SIZE: Vec3 = Vec3::new(10., 8., 6.);
pub const DEFAULT_PARTICLES: UVec3 = UVec3::new(15, 15, 15);

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct Settings {
    pub gravity: Vec3,

    pub dtime: f32,
    pub collision_damping: f32,

    pub smoothing_radius: f32,
    pub target_density: f32,
    pub near_pressure_multiplier: f32,
    pub pressure_multiplier: f32,
    pub mass: f32,

    pub interaction_radius: f32,
    pub interaction_strength: f32,

    pub viscosity_strength: f32,

    pub num_particles: u32,
    pub boundary_particles: u32,
    pub particle_radius: f32,

    pub box_size: Vec3,
    pub _pad: f32,
    pub box_quat: Quat,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dtime: 0.002,

            gravity: vec3(0.0, -9.8, 0.0),
            collision_damping: 0.40,

            smoothing_radius: 0.60,
            target_density: 20.0,

            near_pressure_multiplier: 50.0,
            pressure_multiplier: 500.0,
            viscosity_strength: 0.12,

            interaction_radius: 4.0,
            interaction_strength: 65.0,

            box_size: DEFAULT_BOX_SIZE,
            box_quat: Quat::IDENTITY,
            num_particles: DEFAULT_PARTICLES.x * DEFAULT_PARTICLES.y * DEFAULT_PARTICLES.z,
            boundary_particles: 0,

            mass: 1.0,
            particle_radius: 0.05,
            _pad: 0.0,
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
    pub fn new(p: Vec2, lmb: bool, rmb: bool) -> Self {
        Self {
            position: p,
            clickmask: (lmb as u32) | ((rmb as u32) << 1),
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
    pub translate: Vec3,
    pub z_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct Globals {
    pub view: Mat4,
    pub projection: Mat4,
    pub resolution: UVec2,
    pub _pad: Vec2,
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub const SCALE: f32 = 100.0;
pub const ARRAY_LEN: usize = 262144;
pub const WORKGROUP_SIZE: u32 = 256;
