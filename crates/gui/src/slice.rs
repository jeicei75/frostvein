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

    /// `covered` is whether any solid or ramp tile sits strictly above the cut, which the caller
    /// reads from the mirror. Position alone cannot answer AC10's question: a cut one level under
    /// an empty sky draws the same picture as the surface, so `level == top` said "underground"
    /// for a view that was the surface.
    pub fn label(self, covered: bool) -> &'static str {
        if covered { "underground" } else { "surface" }
    }

    /// `cursor` is the cell under the pointer, and it is here because the client had NO
    /// coordinate feedback of any kind.
    ///
    /// The boot camera is yawed 0.7 rad (~40 degrees), so straight up this client's screen is
    /// world -x +y — a diagonal — while the TUI's screen axes ARE the world axes. The two views
    /// therefore disagree about which way is north by about 135 degrees, and nothing said so.
    /// Wolf hit it on 2026-08-27: dug at what read as north here, found the stone to the west in
    /// the TUI, and reasonably asked whether the coordinates were wrong. They were not. Printing
    /// the cell turns any future cross-client check into a comparison of NUMBERS instead of a
    /// reconciliation of two orientations by eye.
    ///
    /// ASCII only, like the hint bar: the shipped font draws a replacement box for anything else.
    pub fn readout(self, covered: bool, cursor: Option<[i32; 3]>) -> String {
        format!(
            "Slice: z {}/{} - {}  cursor {}",
            self.level,
            self.top,
            self.label(covered),
            match cursor {
                Some([x, y, z]) => format!("{x},{y},{z}"),
                None => "-".to_string(),
            }
        )
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
        assert_eq!(
            slice.readout(false, Some([7, 8, 2])),
            "Slice: z 2/2 - surface  cursor 7,8,2"
        );
        slice.step(-1);
        assert_eq!(
            slice.readout(true, None),
            "Slice: z 1/2 - underground  cursor -",
            "a pointer over nothing must say so rather than print a stale cell"
        );
    }

    /// The readout is drawn in the shipped default font, which has no glyph for most non-ASCII
    /// punctuation and silently renders a replacement BOX instead. This shipped: the separator was
    /// an em-dash, and it has been a box in `7-1-slice.png` and every capture taken since --
    /// reported at 7.2's creation, carried, and never fixed because nothing could fail on it.
    /// A string test cannot see a missing glyph, but it can see the only input that causes one.
    #[test]
    fn the_readout_stays_inside_the_shipped_fonts_glyph_range() {
        let mut slice = SliceLevel::at_world_top(Dims { x: 4, y: 4, z: 32 });
        for _ in 0..33 {
            for covered in [true, false] {
                let readout = slice.readout(covered, None);
                assert!(
                    readout.is_ascii(),
                    "the slice readout must stay ASCII or it draws boxes on the vehicle: \
                     {readout:?} contains {:?}",
                    readout.chars().find(|c| !c.is_ascii()).unwrap()
                );
            }
            slice.step(-1);
        }
    }

    #[test]
    fn the_label_follows_cover_rather_than_position() {
        // The defect this replaced: a cut below the top with nothing above it drew the surface
        // picture and called it underground.
        let mut slice = SliceLevel::at_world_top(Dims { x: 1, y: 1, z: 3 });
        slice.step(-1);
        assert_eq!(
            slice.label(false),
            "surface",
            "nothing above is not underground"
        );
        assert_eq!(slice.label(true), "underground");
        // And the top of the world is not automatically the surface: an overhang can cover it.
        slice.step(99);
        assert_eq!(slice.level(), 2);
        assert_eq!(slice.label(true), "underground");
    }
}
