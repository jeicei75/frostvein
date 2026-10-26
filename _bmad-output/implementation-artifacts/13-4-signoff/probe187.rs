// 13.4 creation probe (throwaway): what the 4 squeeze-unfinished cage runs are doing at 16k.
use std::collections::{BTreeMap, BTreeSet};
use sim_core::{DesignationKind, Dims, Pos, Rect, SimCommand, World};

fn rect(min: Pos, max: Pos) -> Rect { Rect { min, max } }

fn standable(world: &World, pos: Pos) -> bool {
    world.tile(pos) == Some(sim_core::Tile::Empty)
        && matches!(world.tile(Pos { z: pos.z - 1, ..pos }), Some(sim_core::Tile::Solid(_) | sim_core::Tile::Ramp(_)))
}

#[test]
fn probe187() {
    let cases: Vec<(u64, i32)> = std::env::var("CASES").unwrap_or("0:4,6:6,9:6,13:6".into())
        .split(',').map(|c| { let (s, w) = c.split_once(':').unwrap(); (s.parse().unwrap(), w.parse().unwrap()) }).collect();
    for (seed, want) in cases {
        for (west, south, east, north) in [(1, -3, 4, 3), (-1, -5, 4, 4)] {
            let mut world = World::generate(seed, Dims::DEFAULT);
            let camp = world.camp_origin();
            let at = |dx: i32, dy: i32| Pos { x: camp.x + dx, y: camp.y + dy, z: camp.z };
            let channel = rect(at(west, south), at(east, north));
            let width = channel.max.x - channel.min.x + 1;
            if width != want { continue; }
            let pile = (3..12).find_map(|r| {
                [(-r - 2, 0), (0, r), (0, -r - 2)].into_iter()
                    .map(|(dx, dy)| rect(at(dx, dy), at(dx + 2, dy + 2)))
                    .find(|pile| (pile.min.x..=pile.max.x).all(|x| (pile.min.y..=pile.max.y).all(|y| standable(&world, Pos { x, y, z: camp.z }))))
            }).unwrap();
            world.apply_command(SimCommand::Designate { kind: DesignationKind::Channel, rect: channel });
            world.apply_command(SimCommand::PlaceStockpile { rect: pile });
            let mut last_change = 0;
            let mut marks = world.designations().len();
            let mut seen: BTreeMap<u32, BTreeSet<Pos>> = BTreeMap::new();
            let mut claim_changes = [0u32; 5];
            let mut prev_claims = world.claims();
            for _ in 0..16_000 {
                world.step();
                let m = world.designations().len();
                if m != marks { marks = m; last_change = world.tick(); }
                let claims = world.claims();
                if world.tick() > 15_000 {
                    for (id, pos, ..) in world.dwarves() { seen.entry(id.0).or_default().insert(pos); }
                    for i in 0..5 { if claims[i].1 != prev_claims[i].1 { claim_changes[i] += 1; } }
                }
                prev_claims = claims;
            }
            println!("== seed {seed} width {width} marks_left={marks} last_mark_change=t{last_change} pile={pile:?}");
            println!("   marks {:?}", world.designations());
            for job in world.jobs() { println!("   job {:?}", job); }
            let target = world.designations()[0].0;
            let items: BTreeSet<Pos> = world.items().into_iter().map(|(_, p)| p).collect();
            let dw: BTreeSet<Pos> = world.dwarves().into_iter().map(|(_, p, ..)| p).collect();
            for z in [target.z + 1, target.z, target.z - 1] {
                println!("   z={z} (rows y-3..=y+3, cols x-3..=x+3; # rock, . air, _ standable, o item, D dwarf, X target)");
                for y in target.y - 3..=target.y + 3 {
                    let row: String = (target.x - 3..=target.x + 3).map(|x| {
                        let p = Pos { x, y, z };
                        if dw.contains(&p) { 'D' } else if items.contains(&p) { 'o' } else if p == target { 'X' }
                        else if standable(&world, p) { '_' } else { match world.tile(p) { Some(sim_core::Tile::Empty) => '.', Some(sim_core::Tile::Ramp(_)) => '/', _ => '#' } }
                    }).collect();
                    println!("     {row}");
                }
            }
            let carrying = world.carrying();
            for (i, (id, pos, state, _)) in world.dwarves().into_iter().enumerate() {
                println!("   dwarf {} at {:?} {:?} claim={:?} carry={:?} cells_t15k+={} claim_changes={}", id.0, pos, state, world.claims()[i].1, carrying[i].1, seen[&id.0].len(), claim_changes[i]);
            }
        }
    }
}
