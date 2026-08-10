# frostvein

A colony sim: a headless daemon owns the world, clients are thin shells over TCP.

## Setup

```bash
mise install        # Rust 1.97.1, pinned in mise.toml
```

`ripgrep` is the only other requirement, and only for `scripts/gate.sh`.

Every command below works unchanged wherever the repo sits — no environment variables, no
per-machine setup, no separate instructions. Worth knowing why, because the failure it
prevents is baffling: this folder is reachable at two different absolute paths (in the
Nidavellir devpod it is `/workspace/projects/frostvein`; the frostvein devpod mounts it as
`/workspace`), `target/` is shared between them, and Cargo bakes absolute binary paths into
integration-test binaries. Building in one and then testing in the other makes every test
that spawns `simd` or `tui` fail instantly while unit tests stay green. `scripts/gate.sh`
detects the switch and rebuilds the two binary packages itself, so there is nothing to
remember.

If `cargo` is not found, mise activates from an interactive-shell hook that scripts do not
get: `export PATH="$HOME/.cargo/bin:$PATH"`.

## Run

```bash
cargo run -p simd          # prints: listening on 127.0.0.1:7373
cargo run -p simd -- 0     # 0 = OS-assigned port, printed on stdout
```

Then, in another shell, behold it:

```bash
cargo run -p tui           # connects to 127.0.0.1:7373
cargo run -p tui -- 7999   # optional arg: the port simd is listening on
```

| Key | What it does |
| --- | --- |
| `<` / `>` | walk down / up one z-level |
| arrows or `hjkl` | pan the camera |
| `space` | pause / resume |
| `+` / `-` | faster / slower (paused → normal → fast) |
| `d` / `c` | designate a dig / channel rectangle |
| `p` | place a stockpile rectangle |
| `x` | clear designations and stockpiles in a rectangle |
| `Enter` | in a designation mode: mark the first corner, then the second to send it |
| `Esc` | cancel the pending corner, then leave the mode |
| `S` / `L` | save / load the world |
| `q` then `y` | quit (any other key cancels) |
| `Ctrl-C` | quit immediately |

The daemon starts ticking the moment it binds, at a fixed 10 ticks/sec, whether or not
anyone is connected — it does not wait for a client and does not stop when one leaves.
Connect late and you get a snapshot of the world as it is at that moment, then one delta
per tick from there.

You open at the **centre of the map**, on the z-level with the most standable ground.
Terrain is one z-level at a time in 24-bit colour — snow `░`, ice `▒`, soil `▓`, stone `█`,
ramps `▲`, tree trunks `│`, foliage `♠` — with dwarves `☺` on top, torches `†` and a
campfire `♨` beneath them, `☻` where a dwarf shares a cell with a stone and `⚇` where two
dwarves share one. Where a tile is empty the ground up to three levels below shows through,
dimmed with depth. The bottom row reports the tick, the speed, the z-level and the dwarf
count.

**The camp is not on the level you open at.** The dwarves, the campfire and the torches all
sit at z 9 on the shipped seed, while the most-standable-ground rule opens you at z 19 — a
canopy level. You get a forest, and a status line truthfully reporting five dwarves you
cannot see. Pass the level explicitly to find them:

```bash
cargo run -p tui -- --z 9
```

This is a known rough edge in the terminal client, kept deliberately: the opening level is
deterministic (a scripted capture aims where its author thought it did), and the real viewer
is the Bevy client, not this one.

Colour comes from one table, `crates/tui/src/palette.rs`. Nothing on the wire carries RGB.

### Checking the view without a terminal

```bash
cargo run -p tui -- --frame            # one frame from the connect snapshot, exits 0
cargo run -p tui -- --frames 3         # three frames from the LIVE stream, exits 0
cargo run -p tui -- --frames 6 --z 9   # six frames of the camp, which --z is needed to see
```

**Pin `--z`, and take more than one or two frames.** Without `--z` you capture the opening
level, which is not the camp. And dwarves are drawn over the emitters they wander across, so
in any single frame a torch or the campfire may be standing behind a dwarf — the campfire is
hidden in about one frame in nine. Six frames is enough that each glyph appears somewhere in
the capture; one frame is not, and a zero count then means nothing.

Count glyphs with `grep -o '<glyph>' | wc -l`, never `tr -cd`. `tr` works on bytes, and the
box-drawing glyphs share leading UTF-8 bytes, so it reports large counts for glyphs that are
absent entirely.

Rows are newline-terminated, so frames can be piped, captured or diffed. `NO_COLOR` strips
the truecolor entirely — unset it before judging how the world looks.

The difference between the two matters. `--frame` renders the connect snapshot and returns
*before* the reader thread starts, so it can never show a climbing tick. `--frames N` runs
the real client loop and prints one frame per message received, which is what to reach for
when checking that the stream is alive:

```bash
cargo run -p tui -- --frames 3 | grep -o 'tick [0-9]*'
```

Three different numbers means the daemon and the stream are healthy and any problem is in
your terminal; three identical numbers, or a hang, points at the daemon.

To see the raw wire instead — a newline-terminated snapshot (~7.4 MB), then one delta line
per tick, forever:

```bash
bash -c 'head -c 300 < /dev/tcp/127.0.0.1/7373'
```

(`/dev/tcp` is a bash builtin, hence the `bash -c` — it does not work from zsh.)

## Test

```bash
scripts/gate.sh
```

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, a probe that
`tui` has not grown a `sim-core` dependency, and the metrics ledger tests. It exits non-zero,
and `.githooks/pre-commit`
runs it on every commit — enable that once per clone with:

```bash
git config core.hooksPath .githooks
```

## Crates

| Crate | What it is |
| --- | --- |
| `sim-core` | the world, as a pure library — no I/O |
| `protocol` | wire types only; the single home of message shapes |
| `simd` | the daemon: owns the sim, serves it over TCP |
| `tui` | terminal client; talks `protocol` only. Renders what the wire says, holds no game rule |

See `docs/project-brief.md` for what this is, `docs/technical-preferences.md` for how it's built.
