#![no_std]
#![allow(unexpected_cfgs, unused_imports, clippy::too_many_arguments)]

use core::f32;

use gpu_shared::{ARRAY_LEN, Globals, MouseState, Primitive, SCALE, Settings};
use spirv_std::{
    glam::{UVec3, Vec2, Vec3, Vec4, vec2, vec3, vec4},
    num_traits::Float,
    spirv,
};

fn q_rsqrt(value: f32) -> f32 {
    let x2 = value * 0.5;
    let mut y = value;
    let i = y.to_bits();
    let i = 0x5f3759df - (i >> 1);
    y = f32::from_bits(i);
    y = y * (1.5 - (x2 * y * y));

    y
}

// WEBGPU 3d
// +X == RIGHT
// +Y == UP
// +Z == OUT OF SCREEN

pub mod curves;
pub mod gradient;
pub mod sp_hash;

#[spirv(fragment(depth_replacing))]
pub fn fs_main(
    in_view_center: Vec3,
    in_quad: Vec2,
    in_color: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] globals: &Globals,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] settings: &Settings,
    out_color: &mut Vec4,
    #[spirv(frag_depth)] out_depth: &mut f32,
) {
    let r2 = in_quad.dot(in_quad);
    if r2 > 1.0 {
        spirv_std::arch::kill();
    }
    let z = (1.0 - r2).sqrt();
    let normal = vec3(in_quad.x, in_quad.y, z);

    let surface_view = in_view_center + normal * settings.particle_radius;
    let clip = globals.projection * surface_view.extend(1.0);
    *out_depth = clip.z / clip.w;

    // Lambert + ambient
    let light_dir = vec3(0.4, 0.7, 0.5).normalize();
    let intensity = normal.dot(light_dir).max(0.0) * 0.7 + 0.3;

    *out_color = vec4(
        in_color.x * intensity,
        in_color.y * intensity,
        in_color.z * intensity,
        in_color.w,
    );
}

#[spirv(fragment)]
pub fn fs_lines(input: Vec4, output: &mut Vec4) {
    *output = input;
}

#[spirv(vertex)]
pub fn vs_main(
    a_position: Vec2,
    a_prim_id: u32,

    #[spirv(instance_index)] instance_idx: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] globals: &Globals,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] primitives: &[Primitive; ARRAY_LEN],
    #[spirv(position)] out_pos: &mut Vec4,

    out_view_center: &mut Vec3,
    out_quad: &mut Vec2,
    out_color: &mut Vec4,
) {
    let prim = primitives[(a_prim_id + instance_idx) as usize];
    let r = settings.particle_radius;

    let view_center = (globals.view * prim.translate.extend(1.0)).truncate();
    let view_pos = view_center + vec3(a_position.x * r, a_position.y * r, 0.0);

    *out_pos = globals.projection * view_pos.extend(1.0);
    *out_view_center = view_center;
    *out_quad = a_position;
    *out_color = prim.color;
}

#[spirv(vertex)]
pub fn vs_lines(
    a_position: Vec3,
    a_color: Vec3,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] globals: &Globals,
    out_color: &mut Vec4,
) {
    *out_pos = globals.projection * globals.view * a_position.extend(1.0);
    *out_color = a_color.extend(1.0);
}

// Combined external forces and prediction pass
#[spirv(compute(threads(256)))]
pub fn external_forces(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] _mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec4; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles || id < settings.boundary_particles {
        return;
    }

    let idx = id as usize;
    let gravity = settings.gravity;
    let force = gravity;

    velocities[idx] += (force * settings.dtime).extend(0.0);

    // Predict position immediately (combined pass)
    const LOOKAHEAD: f32 = 3.0 / 165.0;
    predictions[idx] = positions[idx] + velocities[idx] * LOOKAHEAD;
}

#[spirv(compute(threads(256)))]
pub fn pre_sort(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;

    starts[idx] = u32::MAX;
    lookup[idx] = id;
    keys[idx] = sp_hash::pos_to_key(
        predictions[idx].truncate(),
        settings.smoothing_radius,
        settings.num_particles,
    );
}

#[spirv(compute(threads(256)))]
pub fn post_sort(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;

    if id == 0 || keys[idx] != keys[idx - 1] {
        starts[keys[idx] as usize] = id;
    }
}

#[spirv(compute(threads(256)))]
pub fn update_densities(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] densities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }
    if id < settings.boundary_particles {
        return;
    }

    let idx = id as usize;
    let my_pos = predictions[idx].truncate();
    let cell = sp_hash::pos_to_cell(my_pos, settings.smoothing_radius);
    let smoothing_radius_sq = settings.smoothing_radius * settings.smoothing_radius;
    let mut density = 0.0;
    let mut near_density = 0.0;

    for neighbor_id in 0..sp_hash::NEIGHBORS.len() {
        let other_cell = cell + sp_hash::NEIGHBORS[neighbor_id];
        let hash = sp_hash::cell_hash(other_cell);
        let key = sp_hash::key_from_hash(hash, settings.num_particles);
        let start = starts[key as usize];

        for i in start..settings.num_particles {
            let particle_key = keys[i as usize];
            if particle_key != key {
                break;
            }

            let other_id = lookup[i as usize];
            let other_idx = other_id as usize;

            let offset = predictions[other_idx].truncate() - my_pos;
            let dist_sq = offset.dot(offset);

            if dist_sq > smoothing_radius_sq {
                continue;
            }

            let dist = dist_sq.sqrt();

            if dist < 0.5 * settings.particle_radius {
                let rng_x = ((id as f32) * 12.9898).sin() * 43758.54;
                let rng_x = rng_x - rng_x.floor();
                let rng_y = ((id as f32) * 78.233).sin() * 43758.54;
                let rng_y = rng_y - rng_y.floor();
                let rng_z = ((id as f32) * 45.164).sin() * 43758.54;
                let rng_z = rng_z - rng_z.floor();
                let random_offset = vec3(rng_x - 0.5, rng_y - 0.5, rng_z - 0.5) * 0.02;
                predictions[idx] += random_offset.extend(0.0);
            }

            let influence = curves::smoothing(dist, settings.smoothing_radius);
            let near_influence = curves::smoothing_near(dist, settings.smoothing_radius);

            density += settings.mass * influence;
            near_density += settings.mass * near_influence;
        }
    }

    densities[idx] = vec2(density, near_density);
}

#[spirv(compute(threads(256)))]
pub fn pressure_force(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] densities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }
    if id < settings.boundary_particles {
        return;
    }

    let idx = id as usize;
    let this_density = densities[idx].x;
    let this_ndensity = densities[idx].y;
    let this_position = predictions[idx].truncate();
    let this_pressure = curves::density_to_pressure(
        this_density,
        settings.target_density,
        settings.pressure_multiplier,
    );
    let this_npressure = this_ndensity * settings.near_pressure_multiplier;

    let cell = sp_hash::pos_to_cell(this_position, settings.smoothing_radius);
    let smoothing_radius_sq = settings.smoothing_radius * settings.smoothing_radius;

    let this_pressure_term = this_pressure / this_density.powi(2);
    let this_npressure_term = this_npressure / this_ndensity.max(f32::EPSILON).powi(2);

    let mut force = Vec3::ZERO;

    for neighbor_id in 0..sp_hash::NEIGHBORS.len() {
        let other_cell = cell + sp_hash::NEIGHBORS[neighbor_id];
        let hash = sp_hash::cell_hash(other_cell);
        let key = sp_hash::key_from_hash(hash, settings.num_particles);
        let start = starts[key as usize];

        for search_id in start..settings.num_particles {
            let search_idx = search_id as usize;

            let particle_key = keys[search_idx];
            if particle_key != key {
                break;
            }

            let other_id = lookup[search_idx];
            let other_idx = other_id as usize;

            if idx == other_idx {
                continue;
            }

            let offset = predictions[other_idx].truncate() - this_position;
            let dist_sq = offset.dot(offset);

            if dist_sq > smoothing_radius_sq || dist_sq < f32::EPSILON {
                continue;
            }

            let inv_dist = q_rsqrt(dist_sq);
            let dist = 1.0 / inv_dist;
            let dir = offset * inv_dist;

            let other_is_boundary = other_id < settings.boundary_particles;

            let other_density: f32;
            let other_ndensity: f32;
            let other_pressure: f32;
            let other_npressure: f32;

            if other_is_boundary {
                other_density = settings.target_density;
                other_ndensity = this_ndensity;
                other_pressure = this_pressure.max(0.0);
                other_npressure = this_npressure;
            } else {
                other_density = densities[other_idx].x;
                other_ndensity = densities[other_idx].y;
                other_pressure = curves::density_to_pressure(
                    other_density,
                    settings.target_density,
                    settings.pressure_multiplier,
                );
                other_npressure = other_ndensity * settings.near_pressure_multiplier;
            }

            let other_pressure_term = other_pressure / other_density.powi(2);
            let other_npressure_term = other_npressure / other_ndensity.max(f32::EPSILON).powi(2);

            // Regular pressure
            let smoothing_term = dir * curves::smoothing_deriv(dist, settings.smoothing_radius);
            let pressure_term = this_pressure_term + other_pressure_term;
            force += settings.mass * pressure_term * smoothing_term;

            // Near pressure
            let nsmoothing_term = dir * curves::nsmoothing_deriv(dist, settings.smoothing_radius);
            let npressure_term = this_npressure_term + other_npressure_term;
            force += settings.mass * npressure_term * nsmoothing_term;
        }
    }

    velocities[idx] += (force * settings.dtime).extend(0.0);
}

#[spirv(compute(threads(256)))]
pub fn viscosity(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }
    if id < settings.boundary_particles {
        return;
    }

    let idx = id as usize;
    let position = predictions[idx].truncate();
    let cell = sp_hash::pos_to_cell(position, settings.smoothing_radius);
    let smoothing_radius_sq = settings.smoothing_radius * settings.smoothing_radius;
    let mut force = Vec3::ZERO;

    for neighbor_id in 0..sp_hash::NEIGHBORS.len() {
        let other_cell = cell + sp_hash::NEIGHBORS[neighbor_id];
        let hash = sp_hash::cell_hash(other_cell);
        let key = sp_hash::key_from_hash(hash, settings.num_particles);
        let start = starts[key as usize];

        for i in start..settings.num_particles {
            let particle_key = keys[i as usize];
            if particle_key != key {
                break;
            }

            let other_id = lookup[i as usize];
            let other_idx = other_id as usize;

            if idx == other_idx {
                continue;
            }

            let offset = predictions[other_idx].truncate() - position;
            let dist_sq = offset.dot(offset);

            if dist_sq > smoothing_radius_sq {
                continue;
            }

            let dist = dist_sq.sqrt();
            let influence = curves::viscosity_smoothing(dist, settings.smoothing_radius);

            let other_velocity = if other_id < settings.boundary_particles {
                Vec3::ZERO
            } else {
                velocities[other_idx].truncate()
            };

            force += (other_velocity - velocities[idx].truncate()) * influence;
        }
    }

    velocities[idx] += (force * settings.viscosity_strength * settings.dtime).extend(0.0);
}

#[spirv(compute(threads(256)))]
pub fn update_positions(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec4; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }
    if id < settings.boundary_particles {
        return;
    }

    let idx = id as usize;
    positions[idx] += velocities[idx] * settings.dtime;
}

#[spirv(compute(threads(256)))]
pub fn collide(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec4; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }
    // Boundary particles live outside [0, size]; don't clamp them back in.
    if id < settings.boundary_particles {
        return;
    }

    let idx = id as usize;
    let size = settings.box_size;
    let rot = settings.box_quat;
    let radius = settings.particle_radius;
    let damping = settings.collision_damping;

    let pos = positions[idx].truncate();
    let vel = velocities[idx].truncate();

    let inv = rot.conjugate();
    let mut lpos = inv * pos;
    let mut lvel = inv * vel;

    if lpos.x < radius {
        lpos.x = radius;
        lvel.x *= -damping;
    } else if lpos.x > size.x - radius {
        lpos.x = size.x - radius;
        lvel.x *= -damping;
    }

    if lpos.y < radius {
        lpos.y = radius;
        lvel.y *= -damping;
    } else if lpos.y > size.y - radius {
        lpos.y = size.y - radius;
        lvel.y *= -damping;
    }

    if lpos.z < radius {
        lpos.z = radius;
        lvel.z *= -damping;
    } else if lpos.z > size.z - radius {
        lpos.z = size.z - radius;
        lvel.z *= -damping;
    }

    let pos = rot * lpos;
    let vel = rot * lvel;

    positions[idx] = pos.extend(0.0);
    velocities[idx] = vel.extend(0.0);
}

#[spirv(compute(threads(256)))]
pub fn copy_prims(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec4; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;
    const MAX_VEL: f32 = 15.0;

    if id < settings.boundary_particles {
        prims[idx].translate = positions[idx].truncate();
        prims[idx].color = vec4(0.2, 0.2, 0.2, 0.25);
        return;
    }

    let speed = velocities[idx].length().clamp(0.0, MAX_VEL);
    let t = speed / MAX_VEL;
    let color = gradient::sample(gradient::VELOCITY, t);

    prims[idx].translate = positions[idx].truncate();
    prims[idx].color = color;
}
