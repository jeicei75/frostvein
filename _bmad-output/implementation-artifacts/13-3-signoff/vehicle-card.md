# 13.3 seat card: the launcher compares full SHAs (AC 6)

Push first (`scripts/push.sh --fast`) so the seat can pull this branch. One sitting, no sim judgement:
the pile-memo is judged by scenario tests and `tick-bench` (M2-26). This card tests only
`scripts/launch-gui.ps1`, which cannot run on the devpod (no `pwsh` there).

**Why the lengths must differ first.** `gui.exe`'s stamp is `git rev-parse --short HEAD` on the devpod,
which prints 8 characters (about 17.5k objects). The Windows checkout printed 7 until the 2026-10-25
workaround set `core.abbrev 8`. With the workaround in place both sides print 8, and a pass proves
nothing. So undo it first.

## Run it

```bash
# WSL: a daemon for the gui to connect to, as usual
simd 7451
```

```powershell
# PowerShell, Windows checkout. Undo the 2026-10-25 workaround so the lengths differ again:
git -C D:\Workspace\frostvein config --unset core.abbrev
git -C D:\Workspace\frostvein rev-parse --short HEAD     # expect 7 characters

# pull this branch (story-13-3-the-pile-remembers-pile-memo), then, with the OLD gui.exe
# (stamped ca862749) still in .bin\:
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')      # RED: expect MISMATCH (a real stale binary)

# rebuild gui.exe at the branch HEAD the usual way, then:
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')      # expect "verified — gui.exe <8 chars>, checkout ..."
```

The verdict prints before the gui starts. Once it has printed, you can close the gui.

**Optional, AC 6's third case.** If you have a `gui.exe` built from a commit the Windows checkout has not
fetched, pass it with `-Exe`:

```powershell
.\scripts\launch-gui.ps1 -Exe D:\path\to\that\gui.exe -GuiArgs @('--subdiv','4')
# expect: the checkout has no single commit for stamp <sha> ... rebuild gui.exe at <head>
```

If none is at hand, skip it; it is recorded below as unseen.

## Questions

1. Did the old binary (stamped `ca862749`) read MISMATCH?
2. Did the rebuilt one read `verified`, with the stamp and the checkout at two different lengths
   (8 and 7)? If both printed the same length, the lengths did not differ and the pass proves nothing.
3. Did a binary from an unfetched commit read "no single commit for stamp"? Or was none at hand (unseen)?

Afterwards you can leave `core.abbrev` unset: the launcher no longer depends on it.

## Sitting

2026-10-25, Wolf, on `46f9afa1`: "so script fix works now". Asked one by one at code review run 1 (2026-10-25):

- Q1, did the old binary read MISMATCH? Q2, did the rebuilt binary read `verified` with two different lengths,
  `core.abbrev` unset? Wolf, to both: "yes it's ok".
- Q3 (a stamp from an unfetched commit): unseen.
