# Story 9.4 — Task 7 vehicle session card

**Vehicle:** gingerspice only (native Windows / NVIDIA Vulkan). This is a pre-session recipe, not
a record of a run. Fill every blank during Epic 9's shared vehicle sitting; do not infer values.

**Merge this into the same sitting as 9.1's card.** Both stories change what the valley floor
looks like and they push its luminance in OPPOSITE directions, so the readings only mean something
if they are taken on the same build.

## 0. Two things that will otherwise look like failures

- **A boot-vista capture may exit 101 BEFORE trees are the cause.** Wolf observed on 2026-08-28
  that 9.1's shadows did not close the campfire blow-out, so `BLOWN_POOL_FRACTION_CEILING`
  (0.6651 %) may already be breached on this branch. **Take the baseline reading first (§1) and
  attribute nothing to trees until you have it.**
- **Fewer trees can only make the blown pool LARGER**, never smaller — dark tree skirts near the
  fire were absorbing light. If the pool grows, that is the measurement, not a regression to tune
  away. **Do not raise the ceiling and do not widen the 70–180 band.**

## 1. Build identity, then the BASELINE reading

```bash
# WSL
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
git rev-parse --short HEAD
./target/debug/simd 7451
```

Copy `gui.exe` across. Its first line must read `gui build <sha>` matching the WSL
`git rev-parse --short HEAD`, with no `-dirty`. Stop if it does not.

**No `--z`**: a cut below the world top skips the calibrated checks, and a run that skips them has
judged nothing. Match the `capture range check:` line by **prefix**, never whole-line.

```bash
gui.exe 7451 --capture 9-4-vista-trees.png --at-tick 20
echo "exit=$?"     # 0, or 101 if the blown-pool ceiling fires (see §0)
```

```text
capture range check: ______________________________________________
  ground-median: ______   (was 123.4 before 9.1/9.4)
  blown-pool: ______ %    (ceiling 0.6651 %)    p99: ______
  exit: ______
```

## 2. The interaction, which is the number this story owes

9.1 pushes the valley floor DOWN, 9.4 pushes it UP. Record which won:

| quantity | before this epic | now | still in bounds? |
| --- | ---: | ---: | :---: |
| ground-median luminance | 123.4 | | 70–180 |
| largest blown pool | 0.9883 % | | ≤ 0.6651 % |

**A breach of either is this story's finding, reported with its number.** The withheld levers
(intensity, amplitude, range, emissive) stay withheld — opening one is Wolf's ruling and belongs in
its own story.

## 3. Wolf's eye (AC10)

Look at the valley at the boot vista and at working zoom.

1. Does it read as **a landscape with trees in it**, rather than a confusion of same-coloured
   blocks? Are trees tellable from the ground at a glance?

   `____________________________________________________________________________`

2. **Is 265 the right number of trees?** The band 230–300 was chosen from a measured curve
   (704 today → 531 → 400 → 265), not from a render. If the valley now reads too sparse or still
   too busy, say which and the knob moves in a follow-up — the roll is one literal at
   `worldgen.rs:184`.

   `____________________________________________________________________________`

3. **Does the green read right at night?** It is `(44,100,58)` under a cool desaturated
   directional, chosen to clear stone by 48 and soil by 50 while keeping blue at or above red. A
   green that separates on the instrument can still look wrong to the eye; that judgement is
   yours and Epic 10's tree pilot can refine it through the bench.

   `____________________________________________________________________________`
