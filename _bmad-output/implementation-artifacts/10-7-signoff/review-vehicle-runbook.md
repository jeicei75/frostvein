# Story 10.7 — post-review vehicle session card (ACs 10-12)

**What this session is for:** UX-DR22's **closing half for ACs 10, 11 and 12 only**. The sun's own
closing half was signed on 2026-09-03 and is not re-opened here — terrain shadows, fps and the
campfire/lantern balance all passed then and nothing in the review patch pass touched them.

**Why a second sitting.** Your first sitting predates ACs 10-12. You sat again after Task 8, and that
is how the incomplete toggles were found — but the torch fix, the emissive fix and the subdiv-2 hole
fix all landed *after* it. Nothing on record is your eye on those three. The review's own "confirmed
by eye" was an agent looking at headless probes.

**What changed since you last sat:**

- **F9 torches** joined F5 sun / F6 campfire / F7 lanterns / F8 ambient, and switching a source off
  now also blacks its **baked emissive face** — the residual glow you found.
- **The subdiv-2 black quads are closed.** They were holes, not shadows: `rgb(5,12,28)`, exactly sky.
- **NEW: `--lights-off`**, a headless-only argument added at your ruling to lift the no-CLI-flag
  decision. It is what lets a test drive the toggles, which is what closes AC11 and AC12 as written.

## Expect these — they are not faults

- **`--capture` exits 101** on the near-white ceiling. The PNG is still written. This breach predates
  the story and is filed as its own defect. **Do not raise the constant.**
- **`touch crates/gui/build.rs` before every build**, or the stamp can lag a commit.

## 1. Build

```bash
export PATH="$HOME/.cargo/bin:$PATH"
git checkout 10-7-the-sun-lights-the-valley && git pull
touch crates/gui/build.rs
cargo build -p simd
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
git rev-parse --short HEAD
./target/debug/simd 7471
```

`gui build`: ____________  `HEAD`: ____________  — must match, must not say `-dirty`

## 2. AC12 — the black quads, at the resolution that had them

```
gui.exe 7471 --subdiv 2
```

Look at the **bases of the pine trunks**, and at the terrace steps in the dig.

- [ ] No hard-edged pure-black quads at the trunk bases.
- [ ] The terrace-step banding reads as shading, not as holes punched through the ground.

Then the shipped default, which must be **unchanged**:

```
gui.exe 7471 --subdiv 1
```

- [ ] Looks as it did at your first sitting. This is the path every one of ACs 1-9 was judged on.

## 3. ACs 10 and 11 — the toggles, including the two you found broken

With `--subdiv 1` running, press each key and watch the on-screen readout follow:

| key | source | what should change |
|---|---|---|
| F5 | sun | the whole valley flattens; shadows go |
| F6 | campfire | the warm pool at the camp dims |
| F7 | lanterns | the moving dwarf-carried warmth goes |
| F8 | ambient | everything not directly lit goes near-black |
| **F9** | **torches** | **the fixed warm points around the camp go — this is the new one** |

- [ ] The readout always names what is actually lit.
- [ ] **Now switch ALL FIVE off.** This is the specific thing you caught last time.
      The camp must be **completely dark — no glow anywhere**, no emitter left in the campfire's
      place. Snowflakes and stars stay visible; they are `unlit` decoration and light nothing.
- [ ] Switch all five back **on**. Everything returns as it was — nothing stays dark.

## 4. If anything above is wrong

Say which line, and stop. Do not adjust a constant to make it look right — the near-white ceiling and
every look constant from 9.1, 9.4, 10.3 and 10.4 are explicitly out of scope here.

## 5. Sign-off

- [ ] **UX-DR22 closing half, ACs 10-12: observed.** Date: __________
- [ ] Or: **not signed** — what failed: ______________________________________
