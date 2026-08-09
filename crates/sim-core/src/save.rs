use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{DesignationKind, Dims, Job, JobState, Pos, Tile};

/// `sim-core`'s complete deterministic state. File I/O belongs to `simd`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub seed: u64,
    pub tick: u64,
    pub dims: Dims,
    pub tiles: Vec<Tile>,
    pub wander_rng: ChaCha8Rng,
    pub next_id: u32,
    pub camp_origin: Pos,
    pub dwarves: Vec<SavedDwarf>,
    pub designations: Vec<(Pos, DesignationKind)>,
    pub zones: Vec<Pos>,
    pub jobs: Vec<Job>,
    pub next_job_id: u32,
    pub items: Vec<(u32, Pos)>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SavedDwarf {
    pub id: u32,
    pub pos: Pos,
    pub state: JobState,
    pub home: Pos,
    pub cooldown: u32,
    pub current_job: Option<u32>,
    pub work_progress: u32,
    pub carrying: Option<u32>,
}
