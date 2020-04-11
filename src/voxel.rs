// TODO(cmc): rendering
// TODO(cmc): collisions (DBVT of DBVTs)
// TODO(cmc): chunk streaming
// TODO(cmc): backface culling
// TODO(cmc): adjacency culling

use crate::maths::prelude::*;
use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
use std::collections::HashMap;

// TODO(cmc): new types for world pos vs. local pos?

// VoxelChunk are identified by the coordinates of their lower-left corner.
struct VoxelChunk {
    // world_pos: Vec3i,
    voxels: [bool; Self::SIZE * Self::SIZE * Self::SIZE],
}

impl VoxelChunk {
    // TODO(cmc): const generics
    pub const SIZE: usize = 32;
}

impl Default for VoxelChunk {
    fn default() -> Self {
        let voxels = [false; Self::SIZE * Self::SIZE * Self::SIZE];
        Self { voxels }
    }
}

impl VoxelChunk {
    pub fn stats(&self /* , passes: &[OptimizationPass] */) -> VoxelModelStats {
        let nb_voxels = self.voxels.iter().map(|&v| v as usize).sum();
        let nb_triangles = nb_voxels * 12;
        VoxelModelStats {
            nb_voxels,
            nb_triangles,
        }
    }

    pub fn iter_local(&self) -> impl Iterator<Item = (Vec3i, bool)> + '_ {
        let (mut x, mut y, mut z) = (0i32, 0i32, 0i32);
        self.voxels.iter().map(move |&v| {
            let res = ((x, y, z).into(), v);

            x += 1;
            if x >= Self::SIZE as i32 {
                x = 0;
                y += 1;
            }
            if y >= Self::SIZE as i32 {
                x = 0;
                y = 0;
                z += 1;
            }

            res
        })
    }

    pub fn iter_world(&self, world_pos: &Vec3i) -> impl Iterator<Item = (Vec3i, bool)> + '_ {
        let world_pos = **world_pos;
        self.iter_local()
            .map(move |(pos, v)| ((*pos + world_pos).into(), v))
    }
}

impl VoxelChunk {
    pub fn checkerboard() -> Self {
        let mut vc = Self::default();
        vc.voxels.iter_mut().enumerate().for_each(|(i, v)| {
            if i % 3 == 0 {
                *v = true;
            }
        });

        vc
    }
}

// -----------------------------------------------------------------------------

pub struct VoxelModel {
    // TODO(cmc): dont use uberly secure hashing methods
    chunks: HashMap<Vec3i, VoxelChunk>,
}

impl Default for VoxelModel {
    fn default() -> Self {
        let chunks = HashMap::with_capacity(8);
        Self { chunks }
    }
}

impl VoxelModel {
    pub fn stats(&self /* , passes: &[OptimizationPass] */) -> VoxelModelStats {
        self.chunks.iter().map(|(_, c)| c.stats(/* passes */)).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Vec3i, bool)> + '_ {
        self.chunks
            .iter()
            .map(|(world_pos, c)| c.iter_world(world_pos))
            .flatten()
    }
}

impl VoxelModel {
    pub fn checkerboard() -> Self {
        let mut chunks = HashMap::with_capacity(1);
        chunks.insert((0, 0, 0).into(), VoxelChunk::checkerboard());

        Self { chunks }
    }

    pub fn from_vox(data: &[u8]) -> AnyResult<Vec<VoxelModel>> {
        let data = dot_vox::load_bytes(data).map_err(|msg| anyhow!("{}", msg))?;
        let models = data
            .models
            .into_iter()
            .map(|vox_model| {
                // A vox model cannot exceed 128x128x128 voxels, i.e. 4 chunks in
                // each direction.
                const VOX_MAX_SIZE: usize = 128;
                const CHUNKS_PER_DIR: usize = VOX_MAX_SIZE / VoxelChunk::SIZE;
                const CHUNKS_TOTAL: usize = CHUNKS_PER_DIR * 3;

                let mut model = {
                    let mut chunks = HashMap::with_capacity(CHUNKS_TOTAL);
                    for x in 0..CHUNKS_PER_DIR {
                        for y in 0..CHUNKS_PER_DIR {
                            for z in 0..CHUNKS_PER_DIR {
                                let pos: Vec3i = (
                                    (x * VoxelChunk::SIZE) as i32,
                                    (y * VoxelChunk::SIZE) as i32,
                                    (z * VoxelChunk::SIZE) as i32,
                                )
                                    .into();
                                chunks.insert(pos, VoxelChunk::default());
                            }
                        }
                    }
                    VoxelModel { chunks }
                };

                for voxel in vox_model.voxels {
                    let pos: Vec3i = (voxel.x as i32, voxel.y as i32, voxel.z as i32).into();
                    model[&WorldPos(pos)] = true;
                }

                model
            })
            .collect();

        Ok(models)
    }
}

// -----------------------------------------------------------------------------

use std::ops::{Index, IndexMut};

// TODO(cmc): IndexOrCreate
// TODO(cmc): clean up copy pastes

struct LocalPos(Vec3i);
struct WorldPos(Vec3i);

impl Index<&WorldPos> for VoxelModel {
    type Output = bool;

    fn index(&self, pos: &WorldPos) -> &Self::Output {
        let pos = pos.0;

        let chunk = {
            let idx = (
                pos.x - pos.x % VoxelChunk::SIZE as i32,
                pos.y - pos.y % VoxelChunk::SIZE as i32,
                pos.z - pos.z % VoxelChunk::SIZE as i32,
            );
            self.chunks.get(&idx.into()).unwrap()
        };

        let voxel = {
            let local_pos: Vec3i = (
                pos.x % VoxelChunk::SIZE as i32,
                pos.y % VoxelChunk::SIZE as i32,
                pos.z % VoxelChunk::SIZE as i32,
            )
                .into();
            let local_idx = local_pos.x
                + local_pos.y * VoxelChunk::SIZE as i32
                + local_pos.z * VoxelChunk::SIZE.pow(2) as i32;
            &chunk.voxels[local_idx as usize]
        };

        voxel
    }
}

impl IndexMut<&WorldPos> for VoxelModel {
    fn index_mut(&mut self, pos: &WorldPos) -> &mut Self::Output {
        let pos = pos.0;

        let chunk = {
            let idx = (
                pos.x - pos.x % VoxelChunk::SIZE as i32,
                pos.y - pos.y % VoxelChunk::SIZE as i32,
                pos.z - pos.z % VoxelChunk::SIZE as i32,
            );
            self.chunks.get_mut(&idx.into()).unwrap()
        };

        let voxel = {
            let local_pos: Vec3i = (
                pos.x % VoxelChunk::SIZE as i32,
                pos.y % VoxelChunk::SIZE as i32,
                pos.z % VoxelChunk::SIZE as i32,
            )
                .into();
            let local_idx = local_pos.x
                + local_pos.y * VoxelChunk::SIZE as i32
                + local_pos.z * VoxelChunk::SIZE.pow(2) as i32;
            &mut chunk.voxels[local_idx as usize]
        };

        voxel
    }
}

// -----------------------------------------------------------------------------

use std::fmt;

impl fmt::Debug for VoxelModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = self.stats();
        f.debug_struct("VoxelModel")
            .field("nb_voxels", &stats.nb_voxels)
            .field("nb_triangles", &stats.nb_triangles)
            .finish()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct VoxelModelStats {
    pub nb_voxels: usize,
    pub nb_triangles: usize,
}

use std::ops::AddAssign;
impl AddAssign for VoxelModelStats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            nb_voxels: self.nb_voxels + other.nb_voxels,
            nb_triangles: self.nb_triangles + other.nb_triangles,
        }
    }
}

use std::iter::Sum;
impl Sum<Self> for VoxelModelStats {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut stats = Self::default();
        iter.for_each(|s| stats += s);

        stats
    }
}
