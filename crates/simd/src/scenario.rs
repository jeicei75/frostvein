//! Scripted runs (13.1): one file replayed headless by `--bench` for tick cost, and served by
//! `--scenario` for a seat run, with the same commands landing on the same ticks in both.
//!
//! A scenario drives the SIM, not the daemon: its commands are `protocol::Command`'s own JSON, and
//! `set_speed`, `save`, `load` and `quit` are refused when the file loads.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, bail};
use serde::Deserialize;

/// Two samples are not a floor; five is the least that shows one.
pub const DEFAULT_RUNS: usize = 5;
/// The tick a seat notices: fast4x has 5 ms, Normal 100.
const SLOW_TICK_MS: f64 = 10.0;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    /// Required, never read: every file says what cost case it exercises and where its cells came from.
    #[serde(rename = "about")]
    _about: String,
    start: Start,
    ticks: u64,
    speed: protocol::Speed,
    commands: Vec<Scheduled>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
enum Start {
    Seed(u64),
    /// Relative to the scenario file's own directory.
    Save(PathBuf),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scheduled {
    /// Ticks after the start tick. A save starts at its own tick, so absolute ticks would not read.
    at: u64,
    command: protocol::Command,
}

pub struct Scenario {
    name: String,
    start: Start,
    ticks: u64,
    speed: protocol::Speed,
    commands: Vec<(u64, protocol::Command)>,
}

impl Scenario {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Self::load_inner(path).with_context(|| format!("scenario {}", path.display()))
    }

    fn load_inner(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path).context("could not read the file")?;
        let File {
            name,
            _about,
            start,
            ticks,
            speed,
            commands,
        } = serde_json::from_str(&text).context("could not parse the file")?;
        if ticks == 0 {
            bail!("ticks must be at least 1");
        }
        if speed == protocol::Speed::Paused {
            bail!("speed paused would never run");
        }
        let mut last = 0;
        for Scheduled { at, command } in &commands {
            if *at >= ticks {
                bail!("a command at {at} is not inside the run's {ticks} ticks");
            }
            if *at < last {
                bail!("commands are out of order: {at} after {last}");
            }
            last = *at;
            if matches!(
                command,
                protocol::Command::SetSpeed { .. }
                    | protocol::Command::Save
                    | protocol::Command::Load
                    | protocol::Command::Quit
            ) {
                bail!("a scenario drives the sim, not the daemon: {command:?} at {at}");
            }
            if let Some(rects) = crate::command_rects(command)
                && (rects.is_empty() || !rects.iter().all(crate::rect_is_valid))
            {
                bail!("an invalid rect in {command:?} at {at}");
            }
        }
        let start = match start {
            Start::Seed(seed) => Start::Seed(seed),
            Start::Save(save) => Start::Save(path.parent().unwrap_or(Path::new("")).join(save)),
        };
        Ok(Self {
            name,
            start,
            ticks,
            speed,
            commands: commands
                .into_iter()
                .map(|Scheduled { at, command }| (at, command))
                .collect(),
        })
    }

    /// A fresh world on the start tick, every time it is called.
    pub fn world(&self) -> anyhow::Result<sim_core::World> {
        match &self.start {
            Start::Seed(seed) => Ok(sim_core::World::generate(*seed, sim_core::Dims::DEFAULT)),
            Start::Save(save) => {
                let path = save.to_str().context("the save path is not UTF-8")?;
                crate::try_load_world_from(path).with_context(|| {
                    format!("scenario {}: could not load {}", self.name, save.display())
                })
            }
        }
    }

    fn schedule(&self, start: u64) -> Schedule {
        Schedule {
            start,
            commands: self.commands.clone(),
            next: 0,
        }
    }
}

/// The commands still to come, handed out once each when their tick arrives.
struct Schedule {
    start: u64,
    commands: Vec<(u64, protocol::Command)>,
    next: usize,
}

impl Schedule {
    fn due(&mut self, tick: u64) -> Vec<protocol::Command> {
        let mut due = Vec::new();
        while let Some((at, command)) = self.commands.get(self.next)
            && self.start + at <= tick
        {
            due.push(command.clone());
            self.next += 1;
        }
        due
    }

    fn applied(&self) -> usize {
        self.next
    }

    fn total(&self) -> usize {
        self.commands.len()
    }
}

/// A scenario being served (`--scenario`): held until a client connects, then run to its end tick.
pub struct Live {
    name: String,
    speed: protocol::Speed,
    end: u64,
    schedule: Schedule,
    holding: bool,
    done: bool,
    client_commands: usize,
}

impl Live {
    pub fn new(scenario: Scenario, start: u64) -> Self {
        Self {
            schedule: scenario.schedule(start),
            name: scenario.name,
            speed: scenario.speed,
            end: start + scenario.ticks,
            holding: true,
            done: false,
            client_commands: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The first client's arrival ends the hold: the speed to run at, once.
    pub fn release(&mut self) -> Option<protocol::Speed> {
        if !self.holding {
            return None;
        }
        self.holding = false;
        println!(
            "scenario {}: client connected, running at {:?}",
            self.name, self.speed
        );
        Some(self.speed)
    }

    /// A client changed the sim during the run, so its end hash need not match the bench's.
    pub fn client_command(&mut self) {
        if !self.done {
            self.client_commands += 1;
        }
    }

    pub fn due(&mut self, tick: u64) -> Vec<protocol::Command> {
        if self.done {
            return Vec::new();
        }
        self.schedule.due(tick)
    }

    /// True once, on the end tick: the caller pauses the world there.
    pub fn finish(&mut self, world: &sim_core::World) -> bool {
        if self.done || world.tick() < self.end {
            return false;
        }
        self.done = true;
        let clients = match self.client_commands {
            0 => String::new(),
            n => format!(" (clients sent {n} commands)"),
        };
        println!(
            "scenario {}: done at tick {} state={:016x} commands={}/{}{clients}",
            self.name,
            world.tick(),
            state_hash(world),
            self.schedule.applied(),
            self.schedule.total()
        );
        true
    }
}

/// FNV-1a 64 over the save encoding: a hash that is the same on every build and every machine,
/// which `std`'s `DefaultHasher` does not promise.
fn state_hash(world: &sim_core::World) -> u64 {
    let bytes = serde_json::to_vec(&world.to_save()).expect("a world always encodes");
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[derive(Debug, PartialEq)]
struct Stats {
    mean: f64,
    p99: f64,
    max: f64,
    slow: usize,
}

/// Step times in ms. p99 is by nearest rank. `ms` is never empty: a scenario runs at least 1 tick.
fn stats(ms: &[f64]) -> Stats {
    let mut sorted = ms.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    Stats {
        mean: sorted.iter().sum::<f64>() / n as f64,
        p99: sorted[(n * 99).div_ceil(100) - 1],
        max: sorted[n - 1],
        slow: sorted.iter().filter(|ms| **ms > SLOW_TICK_MS).count(),
    }
}

/// `simd --bench FILE --runs N`: replay the scenario N times with no TCP and no tick sleep.
///
/// Times `World::step` only. The delta is still built and encoded every tick, as the live loop
/// does, so the world is left exactly as the daemon would leave it (the delta drains the dirty
/// set), but that encoding is outside the timed window.
pub fn bench(path: &Path, runs: usize) -> anyhow::Result<()> {
    let scenario = Scenario::load(path)?;
    let name = &scenario.name;
    let mut means = Vec::with_capacity(runs);
    let mut hashes = Vec::with_capacity(runs);
    let mut end = String::new();
    for run in 1..=runs {
        if cfg!(debug_assertions) {
            eprintln!("DEBUG BUILD: figures are not a measurement");
        }
        let mut world = scenario.world()?;
        let mut schedule = scenario.schedule(world.tick());
        let mut ms = Vec::with_capacity(scenario.ticks as usize);
        for _ in 0..scenario.ticks {
            for command in schedule.due(world.tick()) {
                if let Some(refusal) = crate::apply_sim_command(&mut world, command.clone()) {
                    bail!(
                        "scenario {name}: the sim refused {command:?} at tick {}: {refusal:?}",
                        world.tick()
                    );
                }
            }
            let started = Instant::now();
            world.step();
            ms.push(started.elapsed().as_secs_f64() * 1000.0);
            serde_json::to_string(&crate::bridge::delta(
                &mut world,
                scenario.speed,
                Vec::new(),
            ))?;
        }
        let Stats {
            mean,
            p99,
            max,
            slow,
        } = stats(&ms);
        let hash = state_hash(&world);
        println!(
            "bench {name} run {run}/{runs}: ticks={} commands={}/{} mean={mean:.4} p99={p99:.4} \
             max={max:.3} over10={slow} state={hash:016x}",
            scenario.ticks,
            schedule.applied(),
            schedule.total(),
        );
        means.push(mean);
        hashes.push(hash);
        let goals: Vec<String> = world
            .goals()
            .iter()
            .map(|goal| goal.progress.to_string())
            .collect();
        end = format!(
            "bench {name} end tick={} marks={} items={} zones={} goals={}",
            world.tick(),
            world.designations().len(),
            world.items().len(),
            world.zones().len(),
            goals.join("/")
        );
    }
    println!("{end}");
    let low = means.iter().copied().fold(f64::INFINITY, f64::min);
    let high = means.iter().copied().fold(0.0, f64::max);
    let average = means.iter().sum::<f64>() / runs as f64;
    if hashes.iter().any(|hash| *hash != hashes[0]) {
        let hashes: Vec<String> = hashes.iter().map(|hash| format!("{hash:016x}")).collect();
        bail!(
            "bench {name}: STATE DIVERGED across {runs} runs: {}",
            hashes.join(" ")
        );
    }
    if runs == 1 {
        println!("bench {name}: 1 run, mean {low:.4} ms (1 run: no floor)");
    } else {
        println!(
            "bench {name}: {runs} runs, mean {low:.4}..{high:.4} ms (spread {:.1}%), state identical across {runs} runs",
            (high - low) / average * 100.0
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_take_p99_by_nearest_rank_and_count_slow_ticks() {
        // 1..=200 ms: nearest rank puts p99 at the 198th value, not the max.
        let ms: Vec<f64> = (1..=200).map(f64::from).collect();
        assert_eq!(
            stats(&ms),
            Stats {
                mean: 100.5,
                p99: 198.0,
                max: 200.0,
                slow: 190,
            }
        );
        let one = stats(&[3.0]);
        assert_eq!((one.mean, one.p99, one.max, one.slow), (3.0, 3.0, 3.0, 0));
        let flat = stats(&[0.5; 7]);
        assert_eq!(
            (flat.mean, flat.p99, flat.max, flat.slow),
            (0.5, 0.5, 0.5, 0)
        );
        // Over, not at: a 10 ms tick is not slow.
        assert_eq!(stats(&[10.0, 10.000001]).slow, 1);
    }

    #[test]
    fn the_schedule_hands_out_each_command_once_on_its_tick() {
        let quit = protocol::Command::Quit;
        let mut schedule = Schedule {
            start: 100,
            commands: vec![(0, quit.clone()), (2, quit.clone()), (2, quit.clone())],
            next: 0,
        };
        assert_eq!(schedule.due(100).len(), 1);
        assert_eq!(schedule.due(100).len(), 0, "never twice");
        assert_eq!(schedule.due(101).len(), 0);
        assert_eq!(schedule.due(102).len(), 2);
        assert_eq!((schedule.applied(), schedule.total()), (3, 3));
    }

    #[test]
    fn the_state_hash_follows_the_world() {
        let mut world = sim_core::World::generate(sim_core::DEFAULT_SEED, sim_core::Dims::DEFAULT);
        let before = state_hash(&world);
        assert_eq!(before, state_hash(&world), "the same world hashes the same");
        world.step();
        assert_ne!(
            before,
            state_hash(&world),
            "a stepped world hashes differently"
        );
    }
}
