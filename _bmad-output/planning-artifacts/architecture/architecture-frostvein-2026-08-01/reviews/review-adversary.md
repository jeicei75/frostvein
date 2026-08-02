# Adversarial Review — ARCHITECTURE-SPINE.md (frostvein, 2026-08-01)

Lens: two story-implementing dev agents, working independently and in parallel,
each obeying every AD and convention to the letter — do their units still
integrate? Holes below are only those where BOTH constructions are compliant AND
integration breaks. Silent runtime divergence weighted highest. Missing detail
that the first implementer simply sets as precedent (and the second reads from
the code) is NOT reported.

Note: a prior sequential-fallback pass of this review claimed several fixes
"applied", but the spine as read today (AD-11, conventions table) does not
contain them — treat all fixes below as OPEN until the spine text actually
changes.

Verdict: **the crate/dependency/determinism skeleton is sound, but the spine
admits compliant-yet-incompatible builds in eight places — two of them
silent-desync critical.** All closable with two new ADs and six tightenings.

---

## Hole 1 — AD-9 admits per-kind id counters → wire id collision (CRITICAL)

**Unit A (sim-core "dwarves & spawning" story):** reads AD-9 "assigns each
dwarf/item a monotonically increasing u32 Id" and implements a `DwarfIdAlloc`
starting at 0. Dwarves get ids 0–4. Fully compliant: sim-assigned, monotonic,
u32, stable across save/load.

**Unit B (sim-core "items from digging" story):** same sentence, implements
`ItemIdAlloc` starting at 0. Rocks get ids 0, 1, 2… Also fully compliant —
nothing in AD-9 says the id space is shared across kinds.

**Integration:** the wire keys entities by u32 id (conventions table, singular:
"entity ids u32"). The tui render story keeps `HashMap<u32, Entity>`; each delta
carries "ALL small state in full" (AD-8) — dwarf 3 and rock 3 collide and the
client's entity map silently overwrites one with the other. No compile error, no
panic; the map just lies. The same collision poisons AD-7's "stable Id order"
tie-breaking (two entities compare equal) and `SaveState` entity keying (AD-11).
The escape hatch — "key on `(kind, id)`" — is itself unwritten, so a compliant
tui dev keying on bare id has no reason to composite-key.

**Severity: critical** (silent divergence in client state, save state, and
determinism tie-breaks).

**Fix:** tighten AD-9 — one global monotonically increasing u32 allocator in
`sim-core`; a single id space shared by every entity kind, ids never reused
(including across load). Job ids are a separate named space and never appear
where an entity id is expected.

---

## Hole 2 — `load` wholesale-replaces the world outside the dirty-tile mechanism → connected clients render a dead world (CRITICAL)

**Unit A (sim-core save/load story, AD-11):** implements `World::from_save()` as
*construction* of a fresh world — exactly like worldgen, which also cannot flow
through per-tick dirty tracking (or the first delta after boot is 524k tiles).
AD-8's letter ("the tile grid *mutates* only through `set_tile`") permits direct
construction; even if `set_tile` is used, the per-tick dirty set is empty again
by the time the next tick's delta is assembled. Compliant.

**Unit B (simd delta-assembly story, AD-8):** deltas = dirty tiles + small state
in full, forever; "nothing else is ever diffed". Compliant — AD-8 promises the
dirty set prevents missed-mutation desync.

**Integration:** a client is attached; the player sends `load`. Every tile that
differs between the running world and the loaded one is never reported. The next
delta carries the new entities/designations (small state, in full) on top of the
*old* map — dwarves walk through walls that aren't there. Silent, permanent
desync until an unrelated dig touches each stale tile. Bonus: the tick number
jumps backward and no rule tells the tui whether tick monotonicity may be
assumed.

**Severity: critical** (silent, unbounded desync on a phase-one command).

**Fix:** new AD — *world replacement resnapshots.* Any wholesale replacement of
the world (`load`; future regen) is not a delta-able event: `simd` pushes a
fresh full `snapshot` to every connected client and resumes deltas from the
loaded tick. Same AD: clients treat a `snapshot` at any time as an authoritative
full reset; deltas are contiguous (`tick = last + 1`) between snapshots; a
client seeing a gap reconnects rather than guessing. Also state explicitly that
world *construction* (worldgen, `from_save`) does not dirty-track.

---

## Hole 3 — dig story and haul story each compliantly own "job claiming" → one dwarf, two jobs (HIGH)

**Unit A (sim-core dig-job story):** builds `DigJobs` (FIFO), plus a claiming
system: iterate idle dwarves in stable Id order (AD-7), each claims the oldest
unclaimed dig job; adds a `CurrentDigJob` component. Registers its systems in
the `.chain()`ed schedule. Compliant with AD-7 and "FIFO claiming".

**Unit B (sim-core haul-job story):** builds `HaulJobs`, its *own* claiming
system with the same rules, and its own `CurrentHaulJob` component. Also fully
compliant — no AD says there is one job list, one claiming system, or that a
dwarf holds at most one job.

**Integration:** both systems run every tick; each checks only its own "busy"
component, so an idle dwarf is claimed by a dig job AND a haul job *in the same
tick*. The dwarf pathfinds to two targets on alternating ticks, or one job stays
claimed-but-abandoned forever (designation never completes). Fully deterministic
— the scenario harness happily reproduces the wrong behavior — and no test
either dev writes in isolation catches it. Classic two-owners-of-one-state:
"is this dwarf available?" has no single owner.

**Severity: high** (silent behavioral divergence; determinism masks it).
The prior review pass dismissed job claiming because AD-7 fixes *ordering* —
but ordering is not the attack; ownership is.

**Fix:** new AD — *one job market.* A single job list holds all job kinds as
variants; one claiming system, chained at a fixed point in the schedule; a dwarf
has exactly one optional `CurrentJob` and is claimable iff `None`. FIFO =
ascending job id among unclaimed; dwarves considered in ascending entity id
(AD-7). Job-kind stories add variants and execution systems, never claiming
logic.

---

## Hole 4 — snapshot tile payload has no defined memory order (HIGH)

**Unit A (simd snapshot/encode story):** snapshot carries "dims, tiles"; sending
524k positions is absurd, so the dev defines
`protocol::Snapshot { dims: [u32; 3], tiles: Vec<Tile> }` — flat, index =
`x + y*W + z*W*H`. Compliant: wire type in `protocol` (AD-6), field detail
"owned by the code".

**Unit B (tui render story, built in parallel):** consumes the same struct —
AD-6 guarantees it compiles — and indexes z-fastest, because nothing states the
stride order and, being built simultaneously, neither dev is reading the other's
precedent; each believes they *are* the precedent.

**Integration:** compiles clean, runs clean, renders a shredded map. This is the
one place AD-6's shared-struct guarantee does not help: ordering is invisible to
the type system. Deltas dodge it (dirty tiles carry positions); only the bulk
snapshot is exposed.

**Severity: high** (silent, total; both sides' unit tests pass).

**Fix:** one line in the conventions table — *bulk tile arrays are `[z][y][x]`
row-major, index = `x + y*dims.x + z*dims.x*dims.y`; z = 0 is the bottom
layer.* (Pinning z-orientation also settles which adjacent layer `channel`
affects, and display labels.)

---

## Hole 5 — AD-2 pause + no-ack convention + pure-renderer clients = designating while paused shows nothing (HIGH)

**Unit A (simd tick-loop story):** AD-2: "pause and fast-forward are tick-rate
changes in `simd`" — pause ⇒ rate 0 ⇒ the loop stops calling `tick()`. AD-10:
commands are consumed only at the next tick start, so the queue accumulates.
Compliant on every word.

**Unit B (tui designation story):** AD-4 — clients render what the sim reports,
never compute outcomes; conventions — no ack messages, "a command's effect
appearing in the next delta is the acknowledgement". So the tui does NOT locally
echo the designation; it sends `designate` and waits. Compliant on every word.

**Integration:** the canonical gesture — pause, plan digs, unpause — renders as:
player pauses, drags a designation, and *nothing appears* until unpause. No tick
⇒ no delta ⇒ no ack ⇒ NFR2's ~200 ms bar unboundedly violated by two rules that
each locally satisfy it. Behavioral deadlock both sides ship green; guaranteed
to occur in the demo path.

**Severity: high.**

**Fix:** tighten AD-2 — *the tick loop never stops.* "Pause" is a sim-visible
speed at which the per-tick schedule still runs: command intake (AD-10) and
designation/zone application still execute, the tick counter still increments,
a delta is still emitted; only world-advancing systems (movement, digging,
hauling, job progress) are skipped. Determinism preserved: paused ticks are
ordinary ticks in tick counts and `SaveState`. (Alternative — client-side
pending-designation echo — violates AD-4's spirit and creates a second state
owner; recommend against. Touches an ADOPTED AD, so this goes to Wolf for the
call rather than being closed silently.)

---

## Hole 6 — `designate(rect)` corner semantics: inclusive vs half-open (MEDIUM)

**Unit A (tui designation story):** cursor-drag (3,3)→(5,5), sends
`Rect { min: [3,3,z], max: [5,5,z] }` meaning the 3×3 the player highlighted —
inclusive, the natural reading for cursor cells. Compliant.

**Unit B (sim-core designation-application story):** reads the shared
`protocol::Rect` (compiles fine, AD-6) and iterates `min..max` half-open — the
natural Rust reading. The struct cannot express which is meant. Compliant.

**Integration:** every designation digs one row and one column short of the
player's selection. Self-revealing (the next delta echoes the sim's designation
set, so the tui shows the truth) — hence medium, not high — but it ships, both
sides' tests pass, and each dev will "fix" it in opposite directions.

**Severity: medium.**

**Fix:** conventions table — *wire rects are inclusive of both corners; `min` and
`max` are cells with `min ≤ max` per axis; a single cell is `min == max`.*

---

## Hole 7 — "ALL small state in full" defines the producer, not the consumer: replace vs upsert (MEDIUM)

**Unit A (simd delta story):** sends the complete entity list every delta
(AD-8). An item picked up for hauling (modeled as moving into the dwarf's
inventory, out of the world) simply stops appearing. Compliant.

**Unit B (tui render story):** "applies" each delta by upserting entities by id
into its map — a compliant reading; nothing states absence means deletion.
Ghost rocks accumulate on every hauled tile.

**Severity: medium** (silent divergence; cosmetically obvious eventually, but
"eventually" can be past the demo).

**Fix:** one sentence in AD-8 — *full-resend sections are authoritative
replacements: the client's set for that state kind becomes exactly the list
sent; absence is deletion.*

---

## Hole 8 — open string vocabularies defeat AD-6's compile-time guarantee (MEDIUM)

**Unit A (protocol + simd encode, worldgen side):** conventions say snake_case
values; AD-4 says "typed data (positions, materials, professions)" — the dev
models `material: String` and emits `"granite"`, `"soil"`. Compliant.

**Unit B (tui color-table story):** the mandated id→RGB data table, keyed by
the strings the dev expects: `"stone"`, `"dirt"`, plus a magenta fallback.
Compliant — the convention makes the table `tui`'s to write.

**Integration:** compiles, runs, renders an all-magenta map. AD-6 exists to
prevent divergent shapes, but a `String` field smuggles an unshared vocabulary
through the shared struct. Recurs for every future vocabulary (professions, job
kinds, tile kinds).

**Severity: medium.**

**Fix:** tighten AD-6 — *closed vocabularies (materials, professions, tile
kinds, speeds, command types) are Rust enums in `protocol`, never strings;
`simd` bridges sim-core enums to protocol enums with exhaustive `match`, no
wildcard arm* — drift becomes a compile error, which is AD-6's whole point.

---

## Rejected candidates (recorded so they aren't re-litigated)

- **Dirty-set ownership (drain vs auto-clear):** intra-workspace API; mismatch
  is a compile error against real code the second dev can read. Precedent-
  within-bounds, not a hole.
- **Worldgen flooding the first delta via mandated `set_tile`:** chatty but
  harmless; snapshot-on-connect makes it moot. (Hole 2's "construction does not
  dirty-track" wording settles it anyway.)
- **Reaction-delay hash unpinned (LOW, note only):** a dev reaching for
  `DefaultHasher` gets `RandomState`'s per-process seed and breaks cross-run
  determinism — but the scenario harness (AD-7/AD-10's named enforcement)
  catches it loudly on the first replay. Worth three words in AD-7 ("a fixed,
  named hash — never `RandomState`"), not a real integration hole.
- **Multi-client command arrival order racing:** live-play only; the harness
  drives the lib directly; no determinism promise exists across live sessions.
- **Snapshot torn mid-tick:** Rust's aliasing rules force synchronization —
  worst case a compile error. The contiguity half of the risk is closed by
  Hole 2's fix.
- **Two default ports / save-file I/O owner** (from the prior pass): loud,
  seconds-to-fix, or already forced to a single legal owner by AD-1 + AD-11.
  Below the YAGNI bar for spine text.

## Summary of required spine changes

| # | Change | Closes |
| --- | --- | --- |
| 1 | AD-9: single global id allocator, one id space for all entity kinds; job ids a separate named space | Hole 1 |
| 2 | New AD: world replacement (load) ⇒ full resnapshot to all clients; deltas contiguous between snapshots; construction doesn't dirty-track | Hole 2 |
| 3 | New AD: one job list, one claiming system, one optional `CurrentJob` per dwarf | Hole 3 |
| 4 | Convention: bulk tile arrays `[z][y][x]` row-major, z = 0 bottom | Hole 4 |
| 5 | AD-2: tick loop never stops; pause skips world-advancing systems only (ADOPTED AD — needs Wolf's yes) | Hole 5 |
| 6 | Convention: wire rects inclusive of both corners | Hole 6 |
| 7 | AD-8: full-resend sections are authoritative replacements (absence = deletion) | Hole 7 |
| 8 | AD-6: closed vocabularies are protocol enums, never strings; exhaustive bridge match in simd | Hole 8 |
