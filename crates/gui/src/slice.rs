use bevy::prelude::Resource;
use protocol::Dims;

/// Client-local height filter for the 3D projection. It is deliberately not mirror or wire state.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SliceLevel {
    level: i32,
    top: i32,
}

impl SliceLevel {
    pub fn at_world_top(dims: Dims) -> Self {
        let top = dims.z.saturating_sub(1) as i32;
        Self { level: top, top }
    }

    pub fn pinned(dims: Dims, requested: i32) -> Self {
        let mut slice = Self::at_world_top(dims);
        slice.set(requested);
        slice
    }

    pub fn level(self) -> i32 {
        self.level
    }

    pub fn top(self) -> i32 {
        self.top
    }

    /// Returns whether changing to the requested level altered the projection filter.
    pub fn set(&mut self, requested: i32) -> bool {
        let level = requested.clamp(0, self.top);
        let changed = self.level != level;
        self.level = level;
        changed
    }

    pub fn step(&mut self, delta: i32) -> bool {
        self.set(self.level.saturating_add(delta))
    }

    /// Keeps a retained client-local level valid if a later snapshot changes world dimensions.
    pub fn rebind(&mut self, dims: Dims) -> bool {
        self.top = dims.z.saturating_sub(1) as i32;
        self.set(self.level)
    }

    pub fn label(self) -> &'static str {
        if self.level == self.top {
            "surface"
        } else {
            "underground"
        }
    }

    pub fn readout(self) -> String {
        format!("Slice: z {}/{} — {}", self.level, self.top, self.label())
    }
}

#[cfg(test)]
mod tests {
    use protocol::Dims;

    use super::SliceLevel;

    #[test]
    fn the_slice_starts_at_the_top_and_clamps_at_both_world_bounds() {
        let mut slice = SliceLevel::at_world_top(Dims { x: 1, y: 1, z: 3 });
        assert_eq!(slice.level(), 2);
        assert!(slice.step(-99));
        assert_eq!(slice.level(), 0);
        assert!(!slice.step(-1), "the floor cannot go below zero");
        assert!(slice.step(99));
        assert_eq!(slice.level(), 2);
        assert!(!slice.step(1), "the top cannot go above dims.z - 1");
    }

    #[test]
    fn the_readout_names_the_current_level_and_whether_it_is_surface_or_underground() {
        let mut slice = SliceLevel::at_world_top(Dims { x: 1, y: 1, z: 3 });
        assert_eq!(slice.readout(), "Slice: z 2/2 — surface");
        slice.step(-1);
        assert_eq!(slice.readout(), "Slice: z 1/2 — underground");
    }
}
