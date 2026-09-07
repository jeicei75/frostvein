"""The summariser is a pure function over a CSV, so it is testable HERE.

That matters because the numbers it will summarise are not. This devpod renders through lavapipe,
so its frame timings are a software rasteriser's and mean nothing about the game's performance --
the real logs come from the Windows vehicle. An instrument whose output cannot be checked where it
is written is exactly the shape that manufactures false evidence, so the arithmetic is pinned
against hand-computed values on a synthetic log instead.
"""

import pathlib
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SUMMARY = ROOT / "scripts/bench/perf_summary.py"
sys.path.insert(0, str(ROOT / "scripts/bench"))

import perf_summary  # noqa: E402

HEADER = "frame,t_ms,frametime_ms,terrain,trees,dwarves,dirty_tiles,mark"


def log(rows):
    return "\n".join([HEADER, *rows]) + "\n"


def write(tmp, rows):
    path = tmp / "run.csv"
    path.write_text(log(rows))
    return path


class PercentileTests(unittest.TestCase):
    def test_nearest_rank_percentiles_are_hand_checkable(self):
        # Ten known values, so every answer can be counted off by hand rather than trusted.
        values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 100]
        self.assertEqual(perf_summary.percentile(values, 0.5), 5)
        self.assertEqual(perf_summary.percentile(values, 0.95), 100)
        self.assertEqual(perf_summary.percentile(values, 1.0), 100)
        # Order must not matter; the input is not sorted in the real file.
        self.assertEqual(perf_summary.percentile([100, 3, 1, 2], 0.5), 2)

    def test_a_percentile_of_nothing_is_refused_not_guessed(self):
        with self.assertRaises(perf_summary.PerfError):
            perf_summary.percentile([], 0.5)


class SummariseTests(unittest.TestCase):
    def test_edit_frames_are_summarised_apart_from_steady_ones(self):
        """The whole point of the split: pooled, the edit cost vanishes.

        Steady frames are 10 ms; the two edit frames are 100 ms. Pooled, the median would still be
        10 and the 100s would read as outliers of one population. Split, the edit p50 is 100 -- an
        edit frame costs 10x a steady one, which is the finding.
        """
        rows = [f"{i},{i * 10},10.0,121,265,5,0,0" for i in range(1, 11)]
        rows += [f"{i},{i * 10},100.0,121,265,5,3,0" for i in range(11, 13)]
        summary = perf_summary.summarise(_parse(rows))

        self.assertEqual(summary["steady"]["frames"], 10)
        self.assertEqual(summary["edit"]["frames"], 2)
        self.assertEqual(summary["steady"]["p50"], 10.0)
        self.assertEqual(summary["edit"]["p50"], 100.0)

    def test_frame_zero_is_excluded_because_its_frametime_is_a_placeholder(self):
        """Frame 0 has no predecessor and records 0.0. Pooled it would drag every percentile."""
        rows = ["0,0.0,0.0,121,265,5,0,0"] + [f"{i},{i * 10},10.0,121,265,5,0,0" for i in range(1, 5)]
        summary = perf_summary.summarise(_parse(rows))

        self.assertEqual(summary["frames"], 5, "the row is still counted as recorded")
        self.assertEqual(summary["measured"], 4, "but not as a measurement")
        self.assertEqual(
            summary["steady"]["p50"], 10.0, "a placeholder 0.0 must not enter the percentiles"
        )

    def test_hitches_are_counted_by_ratio_and_by_an_absolute_wall(self):
        """The ratio catches a stutter in a fast run; the wall catches a uniformly awful one."""
        rows = [f"{i},{i * 10},10.0,1,1,1,0,0" for i in range(1, 10)]
        rows.append("10,100,60.0,1,1,1,0,0")  # 6x the median AND over 50ms
        summary = perf_summary.summarise(_parse(rows))

        self.assertEqual(summary["steady"]["hitches_ratio"], 1)
        self.assertEqual(summary["steady"]["hitches_absolute"], 1)

        # A uniformly slow run: nothing is 2x the median, but every frame is over the wall.
        slow = [f"{i},{i * 80},80.0,1,1,1,0,0" for i in range(1, 11)]
        slow_summary = perf_summary.summarise(_parse(slow))
        self.assertEqual(slow_summary["steady"]["hitches_ratio"], 0)
        self.assertEqual(slow_summary["steady"]["hitches_absolute"], 10)

    def test_unchanged_content_is_called_out_rather_than_left_to_be_noticed(self):
        """~140 fps once survived a 39% triangle cut here. A flat timing beside a flat scene is a
        tautology, and the report has to SAY so rather than let a reader assume otherwise."""
        rows = [f"{i},{i * 10},10.0,121,265,5,0,0" for i in range(1, 5)]
        flat = perf_summary.render("run.csv", perf_summary.summarise(_parse(rows)))
        self.assertIn("UNCHANGED", flat)

        moved = [f"{i},{i * 10},10.0,{100 + i},265,5,0,0" for i in range(1, 5)]
        changed = perf_summary.render("run.csv", perf_summary.summarise(_parse(moved)))
        self.assertNotIn("UNCHANGED", changed)
        self.assertIn("terrain=101..104", changed)


    def test_a_marked_frame_is_reported_so_the_moment_can_be_found(self):
        """The mark key's whole value is landmarking a ten-minute log.

        A mark that is recorded but never surfaced is an inert mechanism: the column would move and
        nothing downstream would read it.
        """
        rows = [f"{i},{i * 10},10.0,121,265,5,0,0" for i in range(1, 5)]
        rows.append("5,50,42.5,121,265,5,0,1")
        summary = perf_summary.summarise(_parse(rows))
        self.assertEqual([m["frame"] for m in summary["marked"]], [5])

        report = perf_summary.render("run.csv", summary)
        self.assertIn("MARKED", report)
        self.assertIn("frametime=42.50 ms", report)

        # And an unmarked run says nothing about marks rather than printing an empty heading.
        quiet = perf_summary.render("run.csv", perf_summary.summarise(_parse(rows[:-1])))
        self.assertNotIn("MARKED", quiet)


class LoadTests(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(
            __import__("tempfile").mkdtemp(prefix="frostvein-perf-summary-")
        )

    def tearDown(self):
        __import__("shutil").rmtree(self.tmp, ignore_errors=True)

    def test_a_log_from_a_different_build_is_refused_by_name(self):
        """A silently-accepted column shift is how one column's data reads as another's."""
        path = self.tmp / "run.csv"
        path.write_text("frame,t_ms,frametime_ms\n1,10,10\n")
        with self.assertRaises(perf_summary.PerfError) as caught:
            perf_summary.load(path)
        self.assertIn("expected", str(caught.exception))

    def test_a_header_with_no_rows_is_refused_rather_than_summarised_as_zero(self):
        """EXIT 0 IS NOT A RESULT. A run that recorded nothing must say so, not print zeroes."""
        path = self.tmp / "run.csv"
        path.write_text(HEADER + "\n")
        with self.assertRaises(perf_summary.PerfError):
            perf_summary.load(path)

    def test_the_script_runs_end_to_end_and_reports_both_populations(self):
        rows = [f"{i},{i * 10},10.0,121,265,5,0,0" for i in range(1, 6)]
        rows.append("6,60,90.0,121,265,5,7,0")
        path = write(self.tmp, rows)
        result = subprocess.run(
            [sys.executable, str(SUMMARY), str(path)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("steady-state", result.stdout)
        self.assertIn("edit frames", result.stdout)
        self.assertIn("9.00x the median steady frame", result.stdout)

    def test_a_malformed_row_names_the_line_instead_of_crashing(self):
        path = self.tmp / "run.csv"
        path.write_text(log(["1,10,10,121,265,5,0,0", "2,ten,10,121,265,5,0,0"]))
        with self.assertRaises(perf_summary.PerfError) as caught:
            perf_summary.load(path)
        self.assertIn("line 3", str(caught.exception))

    def test_a_log_holding_only_frame_zero_is_refused_not_summarised_as_a_run(self):
        """The zero-row case was guarded and this one was not -- the same defect one row along.

        Frame 0 carries a placeholder frametime rather than a reading, so a file holding nothing
        else measured NOTHING. It used to print `frames=1 measured=0`, both populations "none
        recorded", and exit 0 -- indistinguishable from a healthy run of a quiet scene.
        """
        path = write(self.tmp, ["0,0.000,0.000,40148,265,5,0,0"])
        with self.assertRaises(perf_summary.PerfError) as caught:
            perf_summary.load(path)
        self.assertIn("nothing was measured", str(caught.exception))

    def test_a_row_with_a_surplus_column_is_refused_like_a_short_one(self):
        """`DictReader` files surplus fields under `None` and drops them without a word.

        A SHORT row already failed on `int(None)`. A LONG one parsed clean and silently lost its
        tail, so the two directions of "wrong shape" behaved differently.
        """
        path = write(self.tmp, ["1,10,10,121,265,5,0,0", "2,20,10,121,265,5,0,0,999"])
        with self.assertRaises(perf_summary.PerfError) as caught:
            perf_summary.load(path)
        self.assertIn("line 3", str(caught.exception))
        self.assertIn("9 fields", str(caught.exception))

    def test_the_run_preamble_is_skipped_by_the_reader_and_echoed_in_the_report(self):
        """AC9. The provenance line must not reach the CSV reader, and must not be swallowed either.

        Without it a frametime cannot be attributed to a build, an asset tree, a subdivision, or a
        vsync state -- and a capped run measures the monitor rather than the scene.
        """
        preamble = (
            "# run: build=abc1234 assets=disk:D:\\Workspace\\frostvein\\assets "
            "subdiv=4 vsync=on terrain=40148 trees=265 dwarves=5"
        )
        path = self.tmp / "run.csv"
        path.write_text(
            HEADER + "\n" + preamble + "\n1,10,10.0,121,265,5,0,0\n2,20,10.0,121,265,5,3,0\n"
        )

        rows = perf_summary.load(path)
        self.assertEqual([row["frame"] for row in rows], [1, 2])
        self.assertEqual(perf_summary.provenance(path), [preamble])

        result = subprocess.run(
            [sys.executable, str(SUMMARY), str(path)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("vsync=on", result.stdout)
        self.assertIn("build=abc1234", result.stdout)

    def test_a_log_with_no_preamble_says_so_rather_than_printing_numbers_alone(self):
        """Silence would read as "this run has no provenance to report", which is not the same."""
        path = write(self.tmp, ["1,10,10.0,121,265,5,0,0", "2,20,10.0,121,265,5,0,0"])
        result = subprocess.run(
            [sys.executable, str(SUMMARY), str(path)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("no run preamble", result.stdout)


def _parse(rows):
    """The row dicts `summarise` expects, without a file round-trip."""
    parsed = []
    for row in rows:
        values = row.split(",")
        parsed.append(
            {
                "frame": int(values[0]),
                "t_ms": float(values[1]),
                "frametime_ms": float(values[2]),
                "terrain": int(values[3]),
                "trees": int(values[4]),
                "dwarves": int(values[5]),
                "dirty_tiles": int(values[6]),
                "mark": int(values[7]) if len(values) > 7 else 0,
            }
        )
    return parsed


if __name__ == "__main__":
    unittest.main()
