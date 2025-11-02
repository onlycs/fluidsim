#![no_std]
#![allow(unexpected_cfgs, unused_imports)]

use core::f32;

use gpu_shared::{ARRAY_LEN, Globals, MouseState, Primitive, SCALE, Settings};
use spirv_std::glam::{UVec3, Vec2, Vec4, vec2, vec4};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

fn q_rsqrt(value: f32) -> f32 {
    let x2 = value * 0.5;
    let mut y = value;
    let i = y.to_bits();
    let i = 0x5f3759df - (i >> 1);
    y = f32::from_bits(i);
    y = y * (1.5 - (x2 * y * y));

    y
}

pub mod curves;
pub mod gradient;
pub mod sp_hash;

#[inline(never)]
#[spirv(fragment)]
pub fn fs_main(input: Vec4, output: &mut Vec4) {
    *output = input;
}

#[spirv(vertex)]
pub fn vs_main(
    a_position: Vec2,
    a_normal: Vec2,
    a_prim_id: u32,

    #[spirv(instance_index)] instance_idx: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] globals: &Globals,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] primitives: &[Primitive; 16384],
    #[spirv(position)] out_pos: &mut Vec4,

    out_color: &mut Vec4,
) {
    let prim = primitives[(a_prim_id + instance_idx) as usize];
    let invert_y = vec2(1.0, -1.0);

    let local_pos = a_position + a_normal;
    let world_pos = local_pos - globals.scroll + prim.translate;
    let transformed_pos = world_pos * globals.zoom / (0.5 * globals.resolution) * invert_y;

    let z = prim.z_index as f32 / 4096.0;
    let position = vec4(transformed_pos.x, transformed_pos.y, z, 1.0);

    *out_pos = position;
    *out_color = prim.color;
}

// Combined external forces and prediction pass
#[spirv(compute(threads(256)))]
pub fn external_forces(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;
    let gravity = vec2(0.0, settings.gravity);
    let mut force = gravity;

    if mouse.left() || mouse.right() {
        let mousepos = mouse.position / SCALE - settings.window_size / SCALE * 0.5;
        let offset = mousepos - positions[idx];
        let dist2 = offset.dot(offset);
        let interaction_radius_sq = settings.interaction_radius * settings.interaction_radius;

        if dist2 < interaction_radius_sq {
            let inv_dist = q_rsqrt(dist2);
            let edge = 1.0 / (inv_dist * settings.interaction_radius);
            let center = 1.0 - edge;
            let dir = offset * inv_dist;
            let strength = settings.interaction_strength * mouse.intensity();

            let gweight = 1.0 - (center * (strength * 0.1).clamp(0.0, 1.0));
            let accel = gravity * gweight + dir * center * strength;

            force = accel - velocities[idx] * center;
        }
    }

    velocities[idx] += force * settings.dtime;

    // Predict position immediately (combined pass)
    const LOOKAHEAD: f32 = 1.0 / 120.0;
    predictions[idx] = positions[idx] + velocities[idx] * LOOKAHEAD;
}

#[spirv(compute(threads(256)))]
pub fn pre_sort(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec2; ARRAY_LEN],
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
        predictions[idx],
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
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec2; ARRAY_LEN],
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

    let idx = id as usize;
    let my_pos = predictions[idx];
    let cell = sp_hash::pos_to_cell(my_pos, settings.smoothing_radius);
    let smoothing_radius_sq = settings.smoothing_radius * settings.smoothing_radius;
    let mut density = 0.0;
    let mut near_density = 0.0;

    for neighbor_id in 0..9 {
        let other_cell = cell + sp_hash::NEIGHBORS[neighbor_id];
        let hash = sp_hash::cell_hash(other_cell);
        let key = sp_hash::key_from_hash(hash, settings.num_particles);
        let mut curr_index = starts[key as usize];

        while curr_index < settings.num_particles {
            let particle_key = keys[curr_index as usize];

            if particle_key != key {
                break;
            }

            let other_id = lookup[curr_index as usize];
            let other_idx = other_id as usize;
            curr_index += 1;

            let offset = predictions[other_idx] - my_pos;
            let dist_sq = offset.dot(offset);

            if dist_sq > smoothing_radius_sq {
                continue;
            }

            let dist = dist_sq.sqrt();

            // Calculate both density and near-density
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
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec2; ARRAY_LEN],
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

    let idx = id as usize;
    let this_density = densities[idx].x;
    let this_ndensity = densities[idx].y;
    let this_position = predictions[idx];
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

    let mut force = vec2(0.0, 0.0);

    for neighbor_id in 0..9 {
        let other_cell = cell + sp_hash::NEIGHBORS[neighbor_id];
        let hash = sp_hash::cell_hash(other_cell);
        let key = sp_hash::key_from_hash(hash, settings.num_particles);
        let mut search_idx = starts[key as usize];

        while search_idx < settings.num_particles
            && let particle_key = keys[search_idx as usize]
            && particle_key == key
        {
            let other_id = lookup[search_idx as usize];
            let other_idx = other_id as usize;
            search_idx += 1;

            if idx == other_idx {
                continue;
            }

            let offset = predictions[other_idx] - this_position;
            let dist_sq = offset.dot(offset);

            if dist_sq > smoothing_radius_sq || dist_sq < f32::EPSILON {
                continue;
            }

            let inv_dist = q_rsqrt(dist_sq);
            let dist = 1.0 / inv_dist;
            let dir = offset * inv_dist;

            let other_density = densities[other_idx].x;
            let other_ndensity = densities[other_idx].y;
            let other_pressure = curves::density_to_pressure(
                other_density,
                settings.target_density,
                settings.pressure_multiplier,
            );
            let other_npressure = other_ndensity * settings.near_pressure_multiplier;
            let other_pressure_term = other_pressure / other_density.powi(2);
            let other_npressure_term = other_npressure / other_ndensity.max(f32::EPSILON).powi(2);

            // Regular pressure
            let smoothing_term = dir * curves::smoothing_deriv(dist, settings.smoothing_radius);
            let pressure_term = this_pressure_term + other_pressure_term;
            force += settings.mass * pressure_term * smoothing_term;

            // Near pressure (prevents clustering)
            let nsmoothing_term = dir * curves::nsmoothing_deriv(dist, settings.smoothing_radius);
            let npressure_term = this_npressure_term + other_npressure_term;
            force += settings.mass * npressure_term * nsmoothing_term;
        }
    }

    velocities[idx] += force * settings.dtime;
}

#[spirv(compute(threads(256)))]
pub fn viscosity(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec2; ARRAY_LEN],
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
    let position = predictions[idx];
    let cell = sp_hash::pos_to_cell(position, settings.smoothing_radius);
    let smoothing_radius_sq = settings.smoothing_radius * settings.smoothing_radius;
    let mut force = vec2(0.0, 0.0);

    for neighbor_id in 0..9 {
        let other_cell = cell + sp_hash::NEIGHBORS[neighbor_id];
        let hash = sp_hash::cell_hash(other_cell);
        let key = sp_hash::key_from_hash(hash, settings.num_particles);
        let mut curr_index = starts[key as usize];

        while curr_index < settings.num_particles {
            let particle_key = keys[curr_index as usize];

            if particle_key != key {
                break;
            }

            let other_id = lookup[curr_index as usize];
            let other_idx = other_id as usize;
            curr_index += 1;

            if idx == other_idx {
                continue;
            }

            let offset = predictions[other_idx] - position;
            let dist_sq = offset.dot(offset);

            if dist_sq > smoothing_radius_sq {
                continue;
            }

            let dist = dist_sq.sqrt();
            let influence = curves::viscosity_smoothing(dist, settings.smoothing_radius);

            force += (velocities[other_idx] - velocities[idx]) * influence;
        }
    }

    velocities[idx] += force * settings.viscosity_strength * settings.dtime;
}

#[spirv(compute(threads(256)))]
pub fn update_positions(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec2; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;
    positions[idx] += velocities[idx] * settings.dtime;
}

#[spirv(compute(threads(256)))]
pub fn collide(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec2; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;
    let half_size = settings.window_size / SCALE * 0.5;

    // Separate if statements instead of if/else to reduce thread divergence
    if positions[idx].y.abs() + settings.particle_radius > half_size.y {
        let sign = 1f32.copysign(positions[idx].y);
        positions[idx].y = half_size.y * sign - settings.particle_radius * sign;
        velocities[idx].y *= -settings.collision_damping;
    }

    if positions[idx].x.abs() + settings.particle_radius > half_size.x {
        let sign = 1f32.copysign(positions[idx].x);
        positions[idx].x = half_size.x * sign - settings.particle_radius * sign;
        velocities[idx].x *= -settings.collision_damping;
    }
}

#[spirv(compute(threads(256)))]
pub fn copy_prims(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    if id >= settings.num_particles {
        return;
    }

    let idx = id as usize;
    const MAX_VEL: f32 = 15.0;

    let speed = velocities[idx].length().clamp(0.0, MAX_VEL);
    let t = speed / MAX_VEL;
    let color = gradient::sample(gradient::VELOCITY, t);

    prims[idx].translate = positions[idx] * SCALE;
    prims[idx].color = color;
}
