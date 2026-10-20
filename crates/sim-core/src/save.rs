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
    // NOTE: deliberately no `#[serde(default)]`, like `items`: a pre-12.12 save fails to decode and
    // simd refuses it. The count is of `GOALS` done, in order; simd refuses one beyond the chain.
    pub goals_completed: u32,
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
    // NOTE: REQUIRED, with no migration (Wolf, 12.11 review run 1): a pre-12.11 save fails to
    // decode and simd logs and refuses the load. serde reads a missing `Option` field as `None`
    // even without `#[serde(default)]`; `deserialize_with` turns that off. Without the field a
    // load restarted a lift or set-down and lost the kept landing.
    #[serde(deserialize_with = "Option::deserialize")]
    pub clearing: Option<SavedClearing>,
}

/// A miner carrying a stone out of his own way (12.11 D6): the stone, and where he sets it down
/// from where, once chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedClearing {
    pub item: u32,
    pub landing: Option<Pos>,
    pub drop_from: Option<Pos>,
}
