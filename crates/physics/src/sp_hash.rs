use gpu_shared::ARRAY_LEN;
use spirv_std::{
    glam::{IVec2, IVec3, Vec2, Vec3, ivec3},
    num_traits::real::Real,
};

pub const NEIGHBORS: [IVec3; 27] = const {
    let mut neighbors = [ivec3(0, 0, 0); 27];
    const OFFSETS: [i32; 3] = [-1, 0, 1];

    let mut x = 0;
    let mut idx = 0;
    while x < 3 {
        let mut y = 0;
        while y < 3 {
            let mut z = 0;
            while z < 3 {
                neighbors[idx] = ivec3(OFFSETS[x], OFFSETS[y], OFFSETS[z]);
                z += 1;
                idx += 1;
            }
            y += 1;
        }
        x += 1;
    }

    neighbors
};

/// Convert position to grid cell coordinates
pub fn pos_to_cell(pos: Vec3, cell_size: f32) -> IVec3 {
    (pos / cell_size).floor().as_ivec3()
}

/// Hash a cell coordinate to a hash value
pub fn cell_hash(IVec3 { x, y, z }: IVec3) -> u32 {
    const P: IVec3 = ivec3(391, 193, 719); // primes
    let hash = (x * P.x) ^ (y * P.y) ^ (z * P.z);
    hash as u32
}

/// Convert hash to key (modulo num_particles for bucket assignment)
pub fn key_from_hash(hash: u32, num_particles: u32) -> u32 {
    hash % num_particles
}

/// Convert cell directly to key
pub fn cell_key(cell: IVec3, num_particles: u32) -> u32 {
    let hash = cell_hash(cell);
    key_from_hash(hash, num_particles)
}

/// Convert position directly to key (convenience function)
pub fn pos_to_key(pos: Vec3, cell_size: f32, num_particles: u32) -> u32 {
    cell_key(pos_to_cell(pos, cell_size), num_particles)
}

/// Get the range of particles for a given key (old approach - kept for
/// compatibility) Note: This is slower than the new while-loop approach in the
/// shaders
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
