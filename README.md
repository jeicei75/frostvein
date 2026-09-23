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
cargo run -p simd          # prints: listening on 127.0.0.1:7451
cargo run -p simd -- 0     # 0 = OS-assigned port, printed on stdout
cargo run -p simd -- 7451 --pause-at 120   # freeze the world on tick 120 for a gui --static-world capture
```

`--pause-at 120` matters when a PERSON starts the client: `gui --static-world` asks for the same
tick-120 freeze, but only reaches the daemon in time if it connects within ~12 s of `simd` starting.

Then, in another shell, behold it:

```bash
cargo run -p tui           # connects to 127.0.0.1:7451
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
bash -c 'head -c 300 < /dev/tcp/127.0.0.1/7451'
```

(`/dev/tcp` is a bash builtin, hence the `bash -c` — it does not work from zsh.)

## Behold it in 3D

`gui` is the Bevy client, and the real viewer — the terminal client above is a 2D instrument
beside it. It depends on `protocol` and `client-core` and holds no game rule: the camera,
the selection and the slice are all client-local, and none of them go on the wire.

```bash
cargo run -p gui           # connects to 127.0.0.1:7451
cargo run -p gui -- 7999   # optional arg: the port simd is listening on
```

**A devpod cannot open a window** — there is no graphics userspace — so a bare `cargo run -p gui`
fails there. Everything below under *Looking without a window* still works: lavapipe gives Bevy a
software Vulkan device, so the client renders and captures headlessly. A real window needs the
Windows vehicle, and there `scripts/launch-gui.ps1` is the way in: it refuses to run a `gui.exe`
whose compiled-in `gui build <sha>` stamp is not the checkout's HEAD, which is the only thing that
has ever reliably caught a stale binary.

### Controls

| Keys | What it does |
| --- | --- |
| `W` `A` `S` `D` | orbit the camera |
| `Q` / `E` | zoom out / in |
| MMB drag | orbit |
| shift + MMB drag | pan — hold `ctrl` as well for 4x |
| wheel | zoom — hold `shift` for 4x |
| `,` / `.` | slice down / up one z-level |
| `C` | print the framing as a pasteable `--camera` line |
| LMB | select the dwarf under the cursor and follow him (with no designate mode armed) |
| `Esc` | release the selection, or abort a designation |
| `1` `2` `3` `4` | designate dig / channel / stockpile / clear — then LMB-drag a rectangle |
| `space` | pause / resume the sim |
| `F1` / `F2` | **Bevy's own** render debug overlay: cycle depth/normal, cycle its opacity |
| `F3` | mark a frame in the perf log |
| `F4` `F5` `F6` `F7` | toggle haze / depth of field / bloom / ambient occlusion — widest-acting first |
| `F8` `F9` `F10` `F11` `F12` | toggle sun / ambient / campfire / torches / lanterns — biggest reach first |

The fps overlay has no key: it is simply on, and `--capture` forces it off so no measured frame
carries it. FXAA has no key either; it is `--fx-off fxaa` only. **F1 and F2 belong to `bevy_dev_tools`**, which
`DefaultPlugins` pulls in automatically — binding anything of ours there means both handlers run,
which is how 11.2 shipped dof and haze onto a debug overlay. `the_client_keymap_avoids_keys_other_plugins_have_claimed`
now fails if any control lands on a reserved key or on another of ours. The keymap is still due a
wider rethink — issue #118.

The slice keys are the **unshifted comma and period**. The on-screen hint calls them `<` / `>`,
which reads as "shift these", and that has already cost one session — see #102, where naming the
controls in the client itself is tracked.

Selecting a dwarf drops the zoom to a readable distance and keeps him centred as he walks; `Esc`
hands the camera back. Nothing above is rebindable, deliberately — there is no rebinding system
until a third concrete need asks for one.

### Writing a framing down, and flying back to it

`C` prints one line naming the whole framing, ending in the `--camera` argument that reproduces it:

```
camera: yaw=0.7 pitch=0.45 distance=45 focus=64,64,9 --camera 0.7,0.45,45,64,64,9
```

**That line IS the save format.** There is no viewpoint registry and no camera-path recorder: paste
the tail of it onto a command line and the rig returns, by exact float comparison. It is what makes
comparing a look change cost one command instead of a hand-flown approximation.

### Looking without a window

```bash
cargo run -p gui -- <port> --headless --static-world --subdiv 4 --frames 160 --capture out.png
```

**Take ~160 frames, not two.** The capture waits for 100 delivered ticks before it fires, so a
short run dies on `capture is black` having written nothing. Every capture also validates what it
drew — warm-lit pixels, the valley floor's median value, near-white area — and a band that trips
panics with exit 101 *after* saving the PNG, naming the framing it was taken at.

| Flag | What it does |
| --- | --- |
| `--headless` | render to an offscreen texture instead of a window |
| `--capture <path>` | save a PNG, validate its ranges, then exit |
| `--frames N` / `--at-tick N` | when to capture |
| `--camera <yaw,pitch,distance,fx,fy,fz>` | open at a framing; works interactively too |
| `--distance <d>` | zoom only, capture only — mutually exclusive with `--camera`, which carries its own |
| `--z <level>` | pin the slice level |
| `--subdiv <n>` | terrain subdivision; defaults to the shipped 4, and the recipes pass it anyway so the frame says what it was |
| `--static-world` | pause the DAEMON for the whole run. Two captures differ only by what you changed **only if** you also pin the flicker and give each capture its own freshly started `simd` — see the note below |
| `--lights-steady` | pin emitter flicker to a fixed phase, so a capture pair is comparable |
| `--fx-off <a,b>` | remove named camera effects: `fxaa`, `ao`, `bloom` |
| `--lights-off <a,b>` | switch named light sources off for a measurement |
| `--perf-log <path>` | write a per-frame CSV |
| `--version` | print `gui build <sha>` and exit |

**`--static-world` pauses the SIM, and that is not the same as a still frame.** It sends the daemon
`SetSpeed { Paused }` and holds the capture until the daemon reports itself stopped. Two things it
does NOT stop, because both run on the client's own wall clock: falling snow (`fall_snow`) and
emitter flicker (`flicker_projection`) — pass `--lights-steady` for the flicker. And the world
freezes at whatever tick it had reached when the client connected, so **captures you intend to
compare must each get a freshly started `simd`**. Measured on `6140ca3`: four captures against
fresh daemons froze at tick 40 every time and the camp window's near-white spread was 0.0070 pp;
five captures sharing one daemon froze 35 ticks apart and spread 0.3619 pp — 52x worse, and past
the bar story 11.1b's AC1 is measured against. This flag was documented as "freeze the sim" while
freezing nothing at all for three stories (issue #105); the sentence above is what it does now.

`--version` is worth using before trusting any frame. The stamp is recomputed on every build, so a
binary that predates your change says so.

## Test

```bash
scripts/gate.sh
```

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, three probes
that `tui`, `client-core` and `gui` have not grown a `sim-core` dependency, the metrics ledger
tests, the bench contract tests, and an audit that every mutation-table row still applies to the
source it names. It exits non-zero, and `.githooks/pre-commit` runs it on every commit — enable
that once per clone with:

```bash
git config core.hooksPath .githooks
git config core.sshCommand 'ssh -o ServerAliveInterval=20 -o ServerAliveCountMax=60'
```

The second line is not optional decoration. Git opens the connection to the remote *before*
running the `pre-push` hook, so the socket sits idle for as long as the hook takes and GitHub
closes it. The keepalives stop that. This cannot be committed into the repo: git deliberately
refuses to let a fetched repository dictate the client's ssh options.

**`pre-push` runs the FAST tier, not the full gate** (changed 2026-09-08, because a multi-minute
hook is not a stricter gate but a flakier one). It skips `simd/tests/serve.rs` and the pixel
guards, and it scopes itself to the range being pushed. **So a green push is NOT full-gate
evidence** — the heavy tier is `scripts/gate.sh` with no arguments, run explicitly, and every
story needs it green before it can be called done.

**Push with the wrapper rather than `git push` directly:**

```bash
scripts/push.sh
```

It gates *before* opening the connection, and then **asks the remote whether the push landed**. That
last step is the point — a push killed by the idle timeout leaves `GATE GREEN` as the last line on
screen and no branch on the remote, which is a failure that reads as a success. See issue #76.

## Crates

| Crate | What it is |
| --- | --- |
| `sim-core` | the world, as a pure library — no I/O |
| `protocol` | wire types only; the single home of message shapes |
| `simd` | the daemon: owns the sim, serves it over TCP |
| `client-core` | the shared client-side world mirror: applies snapshots and deltas, holds no I/O |
| `tui` | terminal client; talks `protocol` only. Renders what the wire says, holds no game rule |
| `gui` | Bevy client, the real viewer; `protocol` and `client-core` only, and equally ruleless |

See `docs/project-brief.md` for what this is, `docs/technical-preferences.md` for how it's built.
