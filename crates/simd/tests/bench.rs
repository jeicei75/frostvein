//! 13.1: `simd --bench` and `simd --scenario`, driven through the real binary.
//!
//! The oracle is independent of the code under test: the expected world is rebuilt here straight
//! through `sim_core` (generate, apply the mark on its tick, step) and hashed by this file's own
//! FNV-1a, so a bench that applied a command late, early, twice or never cannot agree with it.

use std::{
    fs,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

const IO_TIMEOUT: Duration = Duration::from_secs(30);
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/one-dig.json");
/// The fixture's mark: the plan's first tunnel cell, at `at` 5 of 40 ticks.
const DIG: sim_core::Pos = sim_core::Pos {
    x: 73,
    y: 64,
    z: 11,
};
const DIG_AT: u64 = 5;
const TICKS: u64 = 40;

fn fnv(world: &sim_core::World) -> String {
    let bytes = serde_json::to_vec(&world.to_save()).unwrap();
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// What the fixture must reach: DEFAULT_SEED, the dig applied before the step of tick `at`.
fn expected(with_dig: bool) -> String {
    expected_world(sim_core::Dims::DEFAULT, 5, with_dig)
}

fn expected_world(dims: sim_core::Dims, crew: usize, with_dig: bool) -> String {
    let mut world = sim_core::World::generate_with_crew(sim_core::DEFAULT_SEED, dims, crew);
    for _ in 0..TICKS {
        if with_dig && world.tick() == DIG_AT {
            let refused = world.apply_command(sim_core::SimCommand::Designate {
                kind: sim_core::DesignationKind::Dig,
                rect: sim_core::Rect { min: DIG, max: DIG },
            });
            assert_eq!(refused, None, "the oracle's dig was refused");
        }
        world.step();
    }
    fnv(&world)
}

/// A scratch copy of the fixture, edited, in its own directory.
fn variant(name: &str, edit: impl FnOnce(&mut serde_json::Value)) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("frostvein-13-1-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    edit(&mut json);
    let path = dir.join("scenario.json");
    fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    path
}

fn bench(path: &Path, runs: usize) -> Output {
    Command::new(env!("CARGO_BIN_EXE_simd"))
        .args([
            "--bench",
            path.to_str().unwrap(),
            "--runs",
            &runs.to_string(),
        ])
        .output()
        .expect("simd must run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// `state=<16 hex>` from every run line, in order.
fn run_states(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.contains(" run "))
        .map(|line| field(line, "state"))
        .collect()
}

fn field(line: &str, key: &str) -> String {
    line.split_whitespace()
        .find_map(|word| word.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("no {key}= in {line:?}"))
        .to_string()
}

fn end_line(text: &str) -> String {
    text.lines()
        .find(|line| line.contains(" end "))
        .unwrap_or_else(|| panic!("no end line in {text:?}"))
        .to_string()
}

#[test]
fn bench_replays_the_scenario_identically_and_reaches_the_oracle_state() {
    let output = bench(Path::new(FIXTURE), 3);
    let text = stdout(&output);
    assert!(output.status.success(), "bench failed: {output:?}");
    let states = run_states(&text);
    assert_eq!(states.len(), 3, "one line per run: {text}");
    assert!(
        states.iter().all(|state| *state == expected(true)),
        "every run must reach the oracle's world {}: {text}",
        expected(true)
    );
    for line in text.lines().filter(|line| line.contains(" run ")) {
        assert_eq!(field(line, "commands"), "1/1", "{line}");
        assert_eq!(field(line, "ticks"), TICKS.to_string(), "{line}");
    }
    let end = end_line(&text);
    assert_eq!(field(&end, "tick"), TICKS.to_string(), "{end}");
    assert_eq!(field(&end, "marks"), "1", "the dig is still marked: {end}");
    assert!(
        text.contains("one-dig: 3 runs, mean ") && text.contains("state identical across 3 runs"),
        "{text}"
    );
}

#[test]
fn a_scenario_without_its_command_reaches_a_different_world() {
    let path = variant("no-command", |json| {
        json["commands"] = serde_json::json!([])
    });
    let output = bench(&path, 1);
    let _ = fs::remove_dir_all(path.parent().unwrap());
    let text = stdout(&output);
    assert!(output.status.success(), "bench failed: {output:?}");
    assert_eq!(run_states(&text), vec![expected(false)], "{text}");
    assert_ne!(expected(false), expected(true), "the dig must matter");
    assert_eq!(field(&end_line(&text), "marks"), "0");
    let end = end_line(&text);
    assert_eq!(field(&end, "done"), "dig:0,cut:0", "{end}");
    assert_eq!(field(&end, "stored"), "0", "{end}");
    assert_eq!(
        field(&end, "cells"),
        "0",
        "no command, no changed cell: {end}"
    );
    assert!(
        text.contains("commands=0/0") && text.contains("1 run: no floor"),
        "{text}"
    );
}

/// 13.6 AC 7: the end line counts the work done. 400 ticks: creation read the dig still marked at
/// t100 and finished by t200.
#[test]
fn a_finished_dig_is_counted_as_done_and_as_a_changed_cell() {
    let path = variant("long-dig", |json| json["ticks"] = 400.into());
    let output = bench(&path, 1);
    let _ = fs::remove_dir_all(path.parent().unwrap());
    let text = stdout(&output);
    assert!(output.status.success(), "bench failed: {output:?}");
    let end = end_line(&text);
    assert_eq!(field(&end, "done"), "dig:1,cut:0", "{end}");
    assert_eq!(field(&end, "marks"), "0", "the mark is gone: {end}");
    let cells: usize = field(&end, "cells").parse().unwrap();
    assert!(cells >= 1, "a dug cell differs from the start world: {end}");
}

/// 13.6 AC 6: the committed 256² crew-200 scenario does real work. Release only (Q4 a): debug
/// steps this world about 22x slower. `scripts/gate.sh`'s full tier runs it. The first release
/// run (2026-10-30, `f1c83b7a`) read `marks=1478 items=602 zones=1800 done=dig:231,cut:91
/// stored=59 cells=2165` in 91 s, and the same file with `"commands":[]` read
/// `done=dig:0,cut:0 stored=0 cells=0`.
#[test]
#[ignore = "release: steps 256² crew 200 for 2500 ticks"]
fn the_two_hundred_on_256_scenario_does_real_work() {
    if cfg!(debug_assertions) {
        panic!(
            "run with --release: cargo test --release -p simd --test bench -- --ignored two_hundred"
        );
    }
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scenarios/two-hundred-on-256.json"
    ));
    let output = bench(path, 1);
    let text = stdout(&output);
    assert!(output.status.success(), "bench failed: {output:?}");
    let end = end_line(&text);
    let done = field(&end, "done");
    let (dig, cut) = done
        .strip_prefix("dig:")
        .and_then(|rest| rest.split_once(",cut:"))
        .unwrap_or_else(|| panic!("done=dig:N,cut:M in {end}"));
    let count = |text: &str| -> usize { text.parse().unwrap() };
    assert!(count(dig) >= 1, "a dig mark was finished: {end}");
    assert!(count(cut) >= 1, "a cut mark was finished: {end}");
    assert!(
        count(&field(&end, "stored")) >= 1,
        "an item lies on the pile: {end}"
    );
    assert!(
        count(&field(&end, "cells")) >= 1,
        "the world changed: {end}"
    );
}

#[test]
fn every_unusable_scenario_fails_loudly_naming_the_cause() {
    let cases: Vec<(&str, PathBuf, &str)> = vec![
        (
            "missing-save",
            variant("missing-save", |json| {
                json["start"] = serde_json::json!({"save": "nowhere.save"})
            }),
            "nowhere.save",
        ),
        (
            "unknown-type",
            variant("unknown-type", |json| {
                json["commands"][0]["command"]["type"] = "dance".into()
            }),
            "could not parse",
        ),
        (
            "daemon-command",
            variant("daemon-command", |json| {
                json["commands"][0]["command"] = serde_json::json!({"type": "quit"})
            }),
            "not the daemon",
        ),
        (
            "late-command",
            variant("late-command", |json| {
                json["commands"][0]["at"] = TICKS.into()
            }),
            "not inside the run",
        ),
        (
            "out-of-order",
            variant("out-of-order", |json| {
                let dig = json["commands"][0].clone();
                let mut early = dig.clone();
                early["at"] = 1.into();
                json["commands"] = serde_json::json!([dig, early]);
            }),
            "out of order",
        ),
        (
            "refused",
            variant("refused", |json| {
                // The top of the map is air: a dig there marks nothing and is refused.
                json["commands"][0]["command"]["rect"] =
                    serde_json::json!({"min": [10, 10, 31], "max": [10, 10, 31]})
            }),
            "refused",
        ),
        (
            "crew-zero",
            variant("crew-zero", |json| json["start"]["crew"] = 0.into()),
            "start crew",
        ),
        (
            "dims-out-of-range",
            variant("dims-out-of-range", |json| {
                json["start"]["dims"] = serde_json::json!([64, 64, 32])
            }),
            "start dims 64,64,32",
        ),
        (
            "dims-with-save",
            variant(
                "dims-with-save",
                |json| {
                    json["start"] =
                        serde_json::json!({"save": "nowhere.save", "dims": [128, 128, 32]})
                },
            ),
            "start dims and crew",
        ),
        (
            "seed-and-save",
            variant("seed-and-save", |json| {
                json["start"]["save"] = "nowhere.save".into()
            }),
            "seed or a save",
        ),
        (
            "neither-seed-nor-save",
            variant("neither-seed-nor-save", |json| {
                json["start"] = serde_json::json!({})
            }),
            "seed or a save",
        ),
        (
            "crew-too-big",
            variant("crew-too-big", |json| {
                json["start"]["crew"] = 1_000_000.into()
            }),
            "crew 1000000 does not fit",
        ),
        (
            "unknown-field",
            variant("unknown-field", |json| json["seed"] = 1.into()),
            "could not parse",
        ),
    ];
    for (name, path, cause) in cases {
        let output = bench(&path, 1);
        let _ = fs::remove_dir_all(path.parent().unwrap());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{name}: exited 0: {output:?}");
        assert!(
            format!("{stderr:?} {stderr}").contains(cause),
            "{name}: stderr must name {cause:?}: {stderr}"
        );
    }
}

/// 13.6 AC 3: `dims` and `crew` in a seed start are honoured. No command here: the fixture's dig
/// cell is a 128² coordinate, and a world built at the wrong size or crew cannot reach this hash.
#[test]
fn a_seed_start_with_dims_and_crew_builds_that_world() {
    let path = variant("dims-crew", |json| {
        json["commands"] = serde_json::json!([]);
        json["start"]["dims"] = serde_json::json!([256, 256, 32]);
        json["start"]["crew"] = 7.into();
    });
    let output = bench(&path, 1);
    let _ = fs::remove_dir_all(path.parent().unwrap());
    let text = stdout(&output);
    assert!(output.status.success(), "bench failed: {output:?}");
    let want = expected_world(sim_core::Dims::FRESH, 7, false);
    assert_eq!(run_states(&text), vec![want.clone()], "{text}");
    assert_ne!(want, expected(false), "256 and seven must matter");
    assert_ne!(
        want,
        expected_world(sim_core::Dims::DEFAULT, 7, false),
        "the size must matter"
    );
    assert_ne!(
        want,
        expected_world(sim_core::Dims::FRESH, 5, false),
        "the crew must matter"
    );
}

fn lines(reader: impl Read + Send + 'static) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}

/// AC 3: `--scenario` holds the start tick until a client connects, runs, stops on the end tick,
/// and prints the same hash as the bench for the same file.
#[test]
fn a_served_scenario_holds_for_its_client_and_ends_on_the_benchs_hash() {
    let dir = std::env::temp_dir().join(format!("frostvein-13-1-serve-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_simd"))
        .args(["0", "--scenario", FIXTURE])
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("simd must start");
    let out = lines(child.stdout.take().unwrap());
    let mut err = child.stderr.take().unwrap();
    let next = |what: &str| {
        out.recv_timeout(IO_TIMEOUT)
            .unwrap_or_else(|error| panic!("no {what} line: {error:?}"))
    };
    let port: u16 = next("listening")
        .strip_prefix("listening on 127.0.0.1:")
        .expect("the listening line")
        .parse()
        .unwrap();
    assert_eq!(
        next("holding"),
        "scenario one-dig: holding tick 0 until a client connects"
    );
    // Long enough for 100 ticks at fast4x: an unheld daemon would be far past the end.
    thread::sleep(Duration::from_millis(500));
    let stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(IO_TIMEOUT)).unwrap();
    let mut reader = BufReader::new(stream);
    let mut snapshot = String::new();
    reader.read_line(&mut snapshot).unwrap();
    let snapshot: protocol::Snapshot = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(
        snapshot.tick, 0,
        "the world held its start tick for the client"
    );
    // Drained on a thread so the daemon never evicts it; each delta's tick comes back here.
    let (tick_tx, ticks) = mpsc::channel();
    thread::spawn(move || {
        for line in reader.lines().map_while(Result::ok) {
            let delta: protocol::Delta = serde_json::from_str(&line).unwrap();
            if tick_tx.send(delta.tick).is_err() {
                break;
            }
        }
    });
    assert_eq!(
        next("running"),
        "scenario one-dig: client connected, running at Fast4x"
    );
    let done = next("done");
    // Paused on the end tick: once a delta reaches it, the next five (one per 100 ms) stay there.
    let mut after: Vec<u64> = ticks
        .iter()
        .skip_while(|tick| *tick < TICKS)
        .take(6)
        .collect();
    after.remove(0);
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&dir);
    let mut err_text = String::new();
    err.read_to_string(&mut err_text).unwrap();
    // A debug daemon misses fast4x's 5 ms period, so the seat run stretches while the hash still
    // matches; it must say so. Same profile as this test, so a release run asserts its absence.
    assert_eq!(
        err_text.contains("DEBUG BUILD: figures are not a measurement"),
        cfg!(debug_assertions),
        "the served run's debug warning: {err_text:?}"
    );
    let bench_state = run_states(&stdout(&bench(Path::new(FIXTURE), 1)));
    assert_eq!(
        done,
        format!(
            "scenario one-dig: done at tick {TICKS} state={} commands=1/1",
            bench_state[0]
        )
    );
    assert_eq!(bench_state[0], expected(true));
    assert_eq!(after, vec![TICKS; 5], "the world stops on its end tick");
}
