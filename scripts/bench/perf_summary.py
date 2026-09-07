"""Summarise a `gui --perf-log` CSV.

    python3 perf_summary.py <run.csv>

WHY PERCENTILE FRAMETIME AND NOT AVERAGE FPS. Average FPS is a mean of reciprocals, so it
structurally hides the thing a player feels: a run averaging 60 fps with a 100 ms hitch every two
seconds reports "60 fps" and plays badly. The industry summarises frame TIME at percentiles for
exactly this reason. The press convention "1% low" is the same data expressed as FPS and comes in
two incompatible definitions -- the worst 1% of frames averaged, versus the single 99th-percentile
frame -- so this script reports percentiles in milliseconds and names them, and does not print a
"1% low" at all rather than print an ambiguous one.

WHY THE POPULATIONS ARE SPLIT. The expensive frame in this game is not the steady one, it is the
one where a dug tile forces a re-mesh. Averaged together the edit cost disappears into the steady
state -- 10.6 measured everything about a still scene and missed precisely that. Frames are split
on `dirty_tiles`: zero is steady state, anything else is an edit frame, and the two are summarised
separately and never pooled.

Stdlib only, on purpose: Blender runs the uv python, so numpy is invisible to the bench scripts.
"""

import csv
import sys

# A frame is a hitch if it is this many times the median, or over the absolute wall below. The
# ratio catches a stutter in a fast run; the absolute catches a run that is uniformly awful and
# therefore has a median too high for any ratio to fire.
HITCH_RATIO = 2.0
HITCH_ABSOLUTE_MS = 50.0

COLUMNS = ["frame", "t_ms", "frametime_ms", "terrain", "trees", "dwarves", "dirty_tiles", "mark"]


class PerfError(ValueError):
    """The log cannot be summarised, and the reason is concrete."""


def provenance(path):
    """The `# run:` preamble lines, which name the build, asset tree, subdivision and vsync state.

    Returned separately from the rows because they are not rows. A log is read on a different day
    and usually a different machine than it was written on, and a column of frametimes that cannot
    say which build produced it, against which asset tree, at which subdivision, or whether a vsync
    cap meant the run measured the MONITOR, is not a measurement anybody can act on.
    """
    with open(path, newline="") as handle:
        return [line.strip() for line in handle if line.lstrip().startswith("#")]


def load(path):
    """Return the rows, refusing a file whose shape is not the one this script understands."""
    with open(path, newline="") as handle:
        raw_lines = handle.readlines()
    # Comment lines are stripped BEFORE the reader sees them -- `csv.DictReader` would otherwise
    # take a leading `#` line as the header. Original line numbers are carried alongside so a
    # malformed row is still named by its position in the real file.
    body = [
        (number, line)
        for number, line in enumerate(raw_lines, start=1)
        if not line.lstrip().startswith("#")
    ]
    reader = csv.DictReader([line for _, line in body])
    if reader.fieldnames != COLUMNS:
        raise PerfError(
            f"unexpected columns {reader.fieldnames!r}; expected {COLUMNS!r}. A log written by "
            "a different build cannot be compared with one written by this one."
        )
    rows = []
    for index, raw in enumerate(reader):
        # `body[0]` is the header, so the first data row is `body[1]`.
        line = body[index + 1][0]
        # A SHORT row is caught by the `int()` calls below, which see `None` for the missing keys.
        # A LONG one is not: `DictReader` files every surplus field under the `None` key and
        # discards it silently, so a row with a ninth column parsed clean and lost it. The two
        # directions of "wrong shape" must fail the same way.
        if raw.get(None):
            raise PerfError(
                f"line {line} has {len(COLUMNS) + len(raw[None])} fields, expected "
                f"{len(COLUMNS)}; trailing {raw[None]!r}"
            )
        try:
            rows.append(
                {
                    "frame": int(raw["frame"]),
                    "t_ms": float(raw["t_ms"]),
                    "frametime_ms": float(raw["frametime_ms"]),
                    "terrain": int(raw["terrain"]),
                    "trees": int(raw["trees"]),
                    "dwarves": int(raw["dwarves"]),
                    "dirty_tiles": int(raw["dirty_tiles"]),
                    "mark": int(raw["mark"]),
                }
            )
        except (TypeError, ValueError) as error:
            raise PerfError(f"line {line} is malformed: {error}") from error
    if not rows:
        raise PerfError("the log has a header but no rows; the run recorded nothing")
    # Frame 0 carries a placeholder frametime rather than a reading, so a file holding nothing else
    # measured NOTHING. Refused rather than summarised: it used to print `frames=1 measured=0` with
    # both populations "none recorded" and exit 0, which is indistinguishable from a healthy run of
    # a quiet scene. Exit 0 is not a result.
    if all(row["frame"] == 0 for row in rows):
        raise PerfError(
            f"the log has {len(rows)} row(s) but none after frame 0, so nothing was measured; "
            "frame 0 is a placeholder, not a reading"
        )
    return rows


def percentile(values, fraction):
    """Nearest-rank percentile, which needs no interpolation policy to argue about.

    Defined so `percentile(v, 1.0)` is the worst frame and `percentile(v, 0.5)` the median.
    """
    if not values:
        raise PerfError("no frames to take a percentile of")
    ordered = sorted(values)
    rank = max(1, min(len(ordered), -(-int(round(fraction * len(ordered) * 1000)) // 1000)))
    return ordered[rank - 1]


def summarise(rows):
    """Split on `dirty_tiles` and describe each population on its own terms."""
    # The first frame has no predecessor, so its frametime is a placeholder 0 rather than a
    # measurement. Pooling it would drag every percentile down by one sample of pure fiction.
    measured = [row for row in rows if row["frame"] != 0]
    steady = [row for row in measured if row["dirty_tiles"] == 0]
    edit = [row for row in measured if row["dirty_tiles"] > 0]
    return {
        "frames": len(rows),
        "measured": len(measured),
        "duration_ms": rows[-1]["t_ms"] - rows[0]["t_ms"],
        "content": {
            key: (min(r[key] for r in rows), max(r[key] for r in rows))
            for key in ("terrain", "trees", "dwarves")
        },
        "steady": population(steady),
        "edit": population(edit),
        # The frames the operator flagged at the seat. These are the whole point of the mark key:
        # a ten-minute log has no landmark otherwise, and "the bit that felt bad" is unfindable.
        "marked": [
            {"frame": row["frame"], "t_ms": row["t_ms"], "frametime_ms": row["frametime_ms"]}
            for row in rows
            if row["mark"]
        ],
    }


def population(rows):
    if not rows:
        return None
    times = [row["frametime_ms"] for row in rows]
    median = percentile(times, 0.5)
    return {
        "frames": len(rows),
        "p50": median,
        "p95": percentile(times, 0.95),
        "p99": percentile(times, 0.99),
        "p999": percentile(times, 0.999),
        "worst": max(times),
        "hitches_ratio": sum(1 for t in times if median > 0 and t >= median * HITCH_RATIO),
        "hitches_absolute": sum(1 for t in times if t >= HITCH_ABSOLUTE_MS),
    }


def render(path, summary, run=()):
    lines = [f"PERF {path}"]
    # Printed FIRST, because every number below is only meaningful against it.
    lines.extend(f"  {line}" for line in run)
    if not run:
        lines.append("  (no run preamble -- this log cannot say which build or assets produced it)")
    content = summary["content"]
    moved = [key for key, (low, high) in content.items() if low != high]
    lines.append(
        f"  frames={summary['frames']:,} measured={summary['measured']:,} "
        f"over {summary['duration_ms'] / 1000:.1f}s"
    )
    lines.append(
        "  content  "
        + "  ".join(
            f"{key}={low}" if low == high else f"{key}={low}..{high}"
            for key, (low, high) in content.items()
        )
        + ("" if moved else "   (UNCHANGED -- a flat frametime here says nothing)")
    )
    for name, label in (("steady", "steady-state"), ("edit", "edit frames  ")):
        pop = summary[name]
        if pop is None:
            lines.append(f"  {label}  none recorded")
            continue
        lines.append(
            f"  {label}  n={pop['frames']:>6,}  p50={pop['p50']:>7.2f}  p95={pop['p95']:>7.2f}  "
            f"p99={pop['p99']:>7.2f}  p99.9={pop['p999']:>7.2f}  worst={pop['worst']:>8.2f} ms"
        )
        lines.append(
            f"  {' ' * len(label)}  hitches >={HITCH_RATIO:g}x median: {pop['hitches_ratio']:,}"
            f"   >={HITCH_ABSOLUTE_MS:g}ms: {pop['hitches_absolute']:,}"
        )
    if summary["edit"] and summary["steady"]:
        ratio = summary["edit"]["p50"] / summary["steady"]["p50"] if summary["steady"]["p50"] else 0
        lines.append(f"  an edit frame costs {ratio:.2f}x the median steady frame at p50")
    for mark in summary["marked"]:
        lines.append(
            f"  MARKED   frame {mark['frame']:,} at {mark['t_ms'] / 1000:.1f}s  "
            f"frametime={mark['frametime_ms']:.2f} ms"
        )
    return "\n".join(lines)


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__.strip().splitlines()[2].strip())
    path = sys.argv[1]
    try:
        print(render(path, summarise(load(path)), provenance(path)))
    except PerfError as error:
        raise SystemExit(f"perf_summary: {error}") from error


if __name__ == "__main__":
    main()
