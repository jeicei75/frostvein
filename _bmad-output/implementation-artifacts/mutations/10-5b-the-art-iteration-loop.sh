# Story 10.5b sabotage table. Run alone: scripts/mutate.sh <this file>
#
# AC12 asks for at least three rows the run KILLS, one of them AC7's palette loop bound. Every row
# here targets a claim the story makes, not a line that happens to be easy to break.
#
# ANCHOR ON THE ONE SYMBOL, NOT ON ITS NEIGHBOUR. Two of story M2-1's rows matched an ADJACENT PAIR
# of registered systems and stopped applying the moment this story registered a system between them
# -- they pinned nothing, and only the gate's audit said so. Every literal below is chosen to be
# exactly-once on its own.

# AC7, and the row the AC names. The bound used to be the PINES' seven-entry list, so a ten-colour
# dwarf reported seven and his Wood Trunk, Hair and Lantern-flame cells were read by nothing. The
# FIXTURE IS THE DWARF ON PURPOSE: a seven-colour asset reports the same figure whether the reader
# asks the constant or the artifact, so only an asset with a different cell count can discriminate.
mutation "the palette reader is bounded by the pines' seven again" py scripts.tests.test_check_asset.CheckAssetTests.test_the_authored_dwarf_reports_all_ten_cells_not_the_pines_seven <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '    for index in range(CELLS_PER_ROW * CELLS_PER_ROW):\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    for index in range(7):\n'))
PY

# AC7's other half: trimming is what makes ONE reader serve both families. Without it the pines
# report sixteen cells, ten of them black, and the four published literals stop matching.
mutation "the unpainted tail is never trimmed" py scripts.tests.test_check_asset.CheckAssetTests.test_the_four_published_pines_report_their_literal_figures <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = "    while values and values[-1] == UNUSED_CELL:\n        values.pop()\n"
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# AC2/AC3. The resolver's whole job is to change the load prefix; returning the embedded scheme on
# the disk arm means --assets parses, reports, and reads the compiled-in blobs anyway -- the inert
# mechanism, and invisible to anything that only checks the flag was accepted.
mutation "the disk source still loads through embedded://" gui assets_switches_the_scene_source_and_refuses_a_relative_directory <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            SceneSource::Disk(_) => "",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            SceneSource::Disk(_) => "embedded://",\n'))
PY

# AC6. This is the exact defect the story found: a line that reports the same word whatever the
# client read. It stayed green through 10.1 while the bench camera was rolled 110 degrees.
mutation "the startup line reports 'embedded' whatever was read" gui assets_switches_the_scene_source_and_refuses_a_relative_directory <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            SceneSource::Disk(dir) => format!("disk:{}", dir.display()),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            SceneSource::Disk(_) => "embedded".to_string(),\n'))
PY

# AC3's other direction: a relative --assets resolves against get_base_path(), whose last fallback
# is the EXE'S OWN DIRECTORY -- so it would mean one directory in the devpod and another on the
# vehicle, and neither the one that was typed. Accepting it silently is the failure.
mutation "a relative --assets is accepted" gui assets_switches_the_scene_source_and_refuses_a_relative_directory <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            if !dir.is_absolute() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false {\n'))
PY

# AC5. Bevy defaults watching ON once file_watcher is compiled in, so this row is the difference
# between "armed when there is something to watch" and "every headless test in the gate pays for a
# notify thread that can never fire".
mutation "the watcher is armed even with no disk tree" gui the_file_watcher_is_armed_only_when_there_is_a_disk_tree_to_watch <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        watch_for_changes_override: Some(assets.is_some()),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        watch_for_changes_override: Some(true),\n'))
PY

# AC9. Every row must reach disk as it is recorded, because the capture path exits by PANIC and a
# panic runs no destructors. Batching lost 60 of 300 frames on the first real run, and a short file
# reads as a short RUN rather than as a truncated one.
mutation "rows are buffered instead of written" gui every_frame_is_on_disk_the_moment_it_is_recorded <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/perf.rs'); s = p.read_text()
old = '        if let Err(error) = writeln!(file, "{}", row.to_csv())\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if let Err(error) = writeln!(file, "")\n'))
PY

# AC9. frametime is a DELTA between frames. Reporting the time since the log opened instead makes
# every frame look like a monotonically worsening hitch, and the p99 becomes the run length.
mutation "frametime reports elapsed time, not the gap" gui frametime_is_the_gap_between_frames_and_the_first_frame_has_none <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/perf.rs'); s = p.read_text()
old = '            .map_or(0.0, |last| now.duration_since(last).as_secs_f64() * 1000.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .map_or(0.0, |_| now.duration_since(started).as_secs_f64() * 1000.0);\n'))
PY

# AC11. A mark is a LANDMARK, not a mode. Left set, every frame after the key press is flagged and
# the log has no landmark at all -- which is the one thing the key exists to provide.
mutation "a mark latches on instead of flagging one frame" gui a_mark_flags_exactly_the_next_frame <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/perf.rs'); s = p.read_text()
old = '        let mark = std::mem::take(&mut self.pending_mark);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let mark = self.pending_mark;\n'))
PY

# AC10. Pooling the two populations is exactly how 10.6 measured a still scene and missed that one
# dug tile re-meshes the world: the edit cost disappears into the steady state's median.
mutation "steady and edit frames are pooled" py scripts.tests.test_perf_summary.SummariseTests.test_edit_frames_are_summarised_apart_from_steady_ones <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/perf_summary.py'); s = p.read_text()
old = '    edit = [row for row in measured if row["dirty_tiles"] > 0]\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    edit = measured\n'))
PY

# AC10. Frame 0's frametime is a placeholder 0.0, not a measurement. Admitted to the percentiles it
# drags every one of them down by a sample of pure fiction.
mutation "the placeholder first frame enters the percentiles" py scripts.tests.test_perf_summary.SummariseTests.test_frame_zero_is_excluded_because_its_frametime_is_a_placeholder <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/perf_summary.py'); s = p.read_text()
old = '    measured = [row for row in rows if row["frame"] != 0]\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    measured = list(rows)\n'))
PY

# AC10, and the row that guards against this project's own worst instrument shape: ~140 fps once
# survived a 39% triangle cut because the terrain was never rasterised. A flat frametime beside an
# unchanged scene is a tautology, and the report must SAY so rather than leave it to be noticed.
mutation "an unchanged scene is not called out" py scripts.tests.test_perf_summary.SummariseTests.test_unchanged_content_is_called_out_rather_than_left_to_be_noticed <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/perf_summary.py'); s = p.read_text()
old = '        + ("" if moved else "   (UNCHANGED -- a flat frametime here says nothing)")\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        + ""\n'))
PY
