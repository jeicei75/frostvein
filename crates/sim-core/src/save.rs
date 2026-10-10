use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{
    DesignationKind, Dims, Identity, ItemKind, Job, JobState, LightKind, Pos, Profession, Tile,
};

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
    // NOTE: deliberately no `#[serde(default)]` on the kind: a pre-12.7 save fails to decode and
    // simd refuses it.
    pub items: Vec<(u32, Pos, ItemKind)>,
    pub emitters: Vec<(u32, Pos, LightKind)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedDwarf {
    pub id: u32,
    pub pos: Pos,
    pub state: JobState,
    pub home: Pos,
    pub cooldown: u32,
    pub current_job: Option<u32>,
    pub work_progress: u32,
    pub carrying: Option<u32>,
    // NOTE: deliberately no `#[serde(default)]`: a save from before identities existed fails to
    // decode, and simd logs and refuses the load. There is no migration.
    pub identity: Identity,
    // NOTE: deliberately no `#[serde(default)]`, like `identity`: a save from before professions
    // existed fails to decode, and simd logs and refuses the load. There is no migration.
    pub profession: Profession,
    // NOTE: deliberately no `#[serde(default)]`, like `identity`: a pre-12.9 save fails to decode
    // and simd logs and refuses the load. There is no migration. Every dwarf's path is saved, an
    // idle dwarf's exit path included, so a detour or back-off survives a load.
    pub path: Vec<Pos>,
}
