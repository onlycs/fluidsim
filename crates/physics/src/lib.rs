#![no_std]
#![allow(unexpected_cfgs, clippy::too_many_arguments, unused_imports)]

use core::f32;

use gpu_shared::{Globals, MouseState, Primitive, Settings, ARRAY_LEN, SCALE};
use spirv_std::glam::{vec2, vec4, UVec3, Vec2, Vec4};
use spirv_std::num_traits::real::Real;
use spirv_std::spirv;

pub mod curves;
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

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn external_forces(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    let gravity = vec2(0.0, settings.gravity);
    let mut force = gravity;

    if mouse.left() || mouse.right() {
        let mousepos = mouse.position / SCALE - settings.window_size / SCALE / 2.0;
        let offset = mousepos - positions[idx];
        let dist2 = offset.dot(offset);

        if dist2 < settings.interaction_radius.powi(2) {
            let dist = dist2.sqrt();
            let edge = dist / settings.interaction_radius;
            let center = 1. - edge;
            let dir = offset / dist;
            let strength = settings.interaction_strength * mouse.intensity();

            // reduce gravity when interacting with the mouse. makes interaction feel more natural
            let gweight = 1. - (center * (strength / 10.).clamp(0., 1.));

            // the closer you are to mouse, the more you are pulled.
            let accel = gravity * gweight + dir * center * strength;

            force = accel - velocities[idx] * center;
        }
    }

    velocities[idx] += force * settings.dtime;
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn update_predictions(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    const LOOKAHEAD: f32 = 1. / 120.;

    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    predictions[idx] = positions[idx] + velocities[idx] * LOOKAHEAD;
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn pre_sort(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    starts[idx] = u32::MAX;
    if id > settings.num_particles {
        return;
    }

    lookup[idx] = id;
    keys[idx] = sp_hash::pos_to_key(
        predictions[idx],
        settings.smoothing_radius,
        settings.num_particles,
    );
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn post_sort(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    if id == 0 || keys[idx] != keys[idx - 1] {
        starts[keys[idx] as usize] = id;
    }
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn update_densities(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    let cell = sp_hash::pos_to_cell(predictions[idx], settings.smoothing_radius);
    let mut density = 0.0;

    for neighbor_id in 0..9 {
        let other = cell + sp_hash::NEIGHBORS[neighbor_id];
        let key = sp_hash::cell_key(other, settings.num_particles);
        let (begin, end) = sp_hash::get_by_key(key, starts, settings.num_particles);

        for lookup_id in begin..end {
            let other_id = lookup[lookup_id as usize];
            let other_idx = other_id as usize;
            let dist = predictions[idx].distance(predictions[other_idx]);
            let influence = curves::smoothing(dist, settings.smoothing_radius);
            density += settings.mass * influence;
        }
    }

    densities[idx] = density;
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn pressure_force(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    let this_density = densities[idx];
    let this_position = predictions[idx];
    let this_pressure = curves::density_to_pressure(
        this_density,
        settings.target_density,
        settings.pressure_multiplier,
    );

    let cell = sp_hash::pos_to_cell(predictions[idx], settings.smoothing_radius);
    let mut force = vec2(0.0, 0.0);

    for neighbor_id in 0..9 {
        let other = cell + sp_hash::NEIGHBORS[neighbor_id];
        let key = sp_hash::cell_key(other, settings.num_particles);
        let (begin, end) = sp_hash::get_by_key(key, starts, settings.num_particles);

        for lookup_id in begin..end {
            let other_id = lookup[lookup_id as usize];
            let other_idx = other_id as usize;

            if idx == other_idx {
                continue;
            }

            let offset = predictions[other_idx] - this_position;
            let dist = offset.length();

            let dir = if dist <= f32::EPSILON {
                vec2(this_density.cos(), this_density.sin()).normalize()
            } else {
                offset / dist
            };

            let slope = curves::smoothing_deriv(dist, settings.smoothing_radius);
            let pressure = curves::density_to_pressure(
                densities[other_idx],
                settings.target_density,
                settings.pressure_multiplier,
            );
            let pressure_shared = (this_pressure + pressure) / 2.0; // newton's third law

            force += pressure_shared * dir * slope * settings.mass / densities[other_idx];
        }
    }

    velocities[idx] += force / this_density * settings.dtime;
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn update_positions(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    positions[idx] += velocities[idx] * settings.dtime;
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn collide(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    let Vec2 { x, y } = settings.window_size / SCALE / 2.0;

    if positions[idx].y.abs() + settings.particle_radius > y {
        let sign = 1f32.copysign(positions[idx].y);

        positions[idx].y = y * sign + settings.particle_radius * -sign;
        velocities[idx].y *= -settings.collision_damping;
    }

    if positions[idx].x.abs() + settings.particle_radius > x {
        let sign = 1f32.copysign(positions[idx].x);

        positions[idx].x = x * sign + settings.particle_radius * -sign;
        velocities[idx].x *= -settings.collision_damping;
    }
}

#[allow(unused_variables)]
#[spirv(compute(threads(64)))]
pub fn copy_prims(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] settings: &Settings,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] mouse: &MouseState,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] positions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] predictions: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] velocities: &mut [Vec2; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 3)] densities: &mut [f32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] prims: &mut [Primitive; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] lookup: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 1)] starts: &mut [u32; ARRAY_LEN],
    #[spirv(storage_buffer, descriptor_set = 3, binding = 2)] keys: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x;
    let idx = id as usize;

    if id > settings.num_particles {
        return;
    }

    prims[idx].translate = positions[idx] * SCALE;
    prims[idx].color = vec4((densities[idx] / 100.0).clamp(0., 1.), 0.0, 0.0, 1.0);
}
