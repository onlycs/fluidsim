use gpu_shared::ARRAY_LEN;
use spirv_std::glam::{IVec2, Vec2, ivec2};
use spirv_std::num_traits::real::Real;

pub const NEIGHBORS: [IVec2; 9] = [
    ivec2(-1, -1),
    ivec2(-1, 0),
    ivec2(-1, 1),
    ivec2(0, -1),
    ivec2(0, 0),
    ivec2(0, 1),
    ivec2(1, -1),
    ivec2(1, 0),
    ivec2(1, 1),
];

/// Convert position to grid cell coordinates
pub fn pos_to_cell(pos: Vec2, cell_size: f32) -> IVec2 {
    let x = (pos.x / cell_size).floor();
    let y = (pos.y / cell_size).floor();
    ivec2(x as i32, y as i32)
}

/// Hash a cell coordinate to a hash value
pub fn cell_hash(IVec2 { x, y }: IVec2) -> u32 {
    const PX: i32 = 17;
    const PY: i32 = 31;
    // Use wrapping operations to avoid overflow issues
    let h = x.wrapping_mul(PX).wrapping_add(y.wrapping_mul(PY));
    h as u32
}

/// Convert hash to key (modulo num_particles for bucket assignment)
pub fn key_from_hash(hash: u32, num_particles: u32) -> u32 {
    hash % num_particles
}

/// Convert cell directly to key
pub fn cell_key(cell: IVec2, num_particles: u32) -> u32 {
    let hash = cell_hash(cell);
    key_from_hash(hash, num_particles)
}

/// Convert position directly to key (convenience function)
pub fn pos_to_key(pos: Vec2, cell_size: f32, num_particles: u32) -> u32 {
    cell_key(pos_to_cell(pos, cell_size), num_particles)
}

/// Get the range of particles for a given key (old approach - kept for compatibility)
/// Note: This is slower than the new while-loop approach in the shaders
pub fn get_by_key(key: u32, starts: &[u32; ARRAY_LEN], num_particles: u32) -> (u32, u32) {
    if key >= num_particles {
        return (0, 0);
    }

    let idx = starts[key as usize];
    if idx == u32::MAX {
        return (0, 0);
    }

    let mut end = num_particles;
    for i in key + 1..end {
        if starts[i as usize] != u32::MAX {
            end = starts[i as usize];
            break;
        }
    }

    if end <= idx {
        return (0, 0);
    }

    (idx, end)
}
