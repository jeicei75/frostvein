# frostvein

A colony sim: a headless daemon owns the world, clients are thin shells over TCP.

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
| `q` then `y` | quit (any other key cancels) |
| `Ctrl-C` | quit immediately |

You open centred on a dwarf. Terrain is one z-level at a time in 24-bit colour — snow `░`,
ice `▒`, soil `▓`, stone `█`, ramps `▲`, dwarves `☺` — and where a tile is empty the ground
up to three levels below shows through, dimmed with depth. The bottom row reports the
z-level, camera, dwarf count and keys.

Colour comes from one table, `crates/tui/src/palette.rs`. Nothing on the wire carries RGB.

### Checking the view without a terminal

```bash
cargo run -p tui -- --frame            # one frame to stdout, no raw mode, exits 0
cargo run -p tui -- --frame | head -45
```

Rows are newline-terminated, so the frame can be piped, captured or diffed. Note that
`NO_COLOR` strips the truecolor entirely — unset it before judging how the world looks.

To see the raw wire instead — one newline-terminated JSON line (~6.9 MB), then silence:

```bash
bash -c 'head -c 300 < /dev/tcp/127.0.0.1/7373'
```

(`/dev/tcp` is a bash builtin, hence the `bash -c` — it does not work from zsh.)

## Test

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Crates

| Crate | What it is |
| --- | --- |
| `sim-core` | the world, as a pure library — no I/O |
| `protocol` | wire types only; the single home of message shapes |
| `simd` | the daemon: owns the sim, serves it over TCP |
| `tui` | terminal client; talks `protocol` only. Renders what the wire says, holds no game rule |

See `docs/project-brief.md` for what this is, `docs/technical-preferences.md` for how it's built.
