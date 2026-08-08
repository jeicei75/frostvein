//! The depth view: one ray per map cell, stepped through the voxel grid the client
//! already holds. No second camera, no second world, no second colour table.

/// The tile step for a heading. Index 0 is `+x` ("east"), then clockwise on screen —
/// screen `+y` is south, so the sequence is e, se, s, sw, w, nw, n, ne.
///
/// An integer table rather than a yaw angle: `ViewState` must stay `Copy + Eq`, and a
/// scripted `--key` capture must render byte-identically on every run.
pub fn heading_step(heading: u8) -> (i64, i64) {
    match heading % 8 {
        0 => (1, 0),
        1 => (1, 1),
        2 => (0, 1),
        3 => (-1, 1),
        4 => (-1, 0),
        5 => (-1, -1),
        6 => (0, -1),
        7 => (1, -1),
        _ => unreachable!("modulo 8"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heading_table_is_pinned_clockwise_from_east() {
        // Hand-written truth, not derived from the function under test: index 0 faces
        // +x and each step turns 45 degrees clockwise on screen (+y is south).
        let expected = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        for (heading, step) in expected.iter().enumerate() {
            let heading = heading as u8;
            assert_eq!(heading_step(heading), *step, "step for heading {heading}");
        }
    }

    #[test]
    fn every_step_is_a_distinct_unit_move() {
        let mut steps = std::collections::BTreeSet::new();
        for heading in 0..8 {
            let (dx, dy) = heading_step(heading);
            assert!((-1..=1).contains(&dx) && (-1..=1).contains(&dy));
            assert!((dx, dy) != (0, 0), "heading {heading} does not move");
            assert!(steps.insert((dx, dy)), "heading {heading} repeats a step");
        }
        assert_eq!(steps.len(), 8);
    }

    #[test]
    fn opposite_headings_cancel() {
        for heading in 0..8u8 {
            let (dx, dy) = heading_step(heading);
            let (bx, by) = heading_step((heading + 4) % 8);
            assert_eq!((dx + bx, dy + by), (0, 0), "heading {heading}");
        }
    }
}
