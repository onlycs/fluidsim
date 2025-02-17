#![no_std]
#![allow(unexpected_cfgs, clippy::too_many_arguments, unused_imports)]

use gpu_shared::{Globals, MouseState, Primitive, Settings, ARRAY_LEN, SCALE};
use spirv_std::glam::{vec2, vec4, UVec3, Vec2, Vec4};
use spirv_std::num_traits::real::Real;
use spirv_std::spirv;

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
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] starts: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x as usize;

    if id > settings.num_particles as usize {
        return;
    }

    let gravity = vec2(0.0, settings.gravity);
    let mut force = gravity;

    if mouse.left() || mouse.right() {
        let mousepos = mouse.position / SCALE - settings.window_size / SCALE / 2.0;
        let offset = mousepos - positions[id];
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

            force = accel - velocities[id] * center;
        }
    }

    velocities[id] += force * settings.dtime;
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
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] starts: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x as usize;

    if id > settings.num_particles as usize {
        return;
    }

    positions[id] += velocities[id] * settings.dtime;
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
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)] starts: &mut [u32; ARRAY_LEN],

    #[spirv(global_invocation_id)] id: UVec3,
) {
    let id = id.x as usize;

    if id > settings.num_particles as usize {
        return;
    }

    prims[id].translate = positions[id] * SCALE;
    prims[id].color = vec4(1.0, 1.0, 1.0, 1.0);
}
