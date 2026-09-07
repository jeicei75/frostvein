<#
.SYNOPSIS
    Launch gui.exe against the checkout it was built from — refusing if the two are not the same
    commit. Run it with no arguments.

.DESCRIPTION
    Story 10.5b Task 4; closes issue #46 (M2-7, "automate the gui.exe rebuild-and-copy — the stamp
    half is done").

    THE CHECK IS THE POINT, NOT THE COPYING. A launcher that only copies is a convenience. This one
    exists because `--assets` introduces a SECOND candidate asset tree, and two candidate trees is
    the stale-artifact shape this project keeps paying for: the binary is built in a devpod and
    dropped here, the checkout is fetched separately, and nothing but this script makes them one
    act. A session reading a dwarf from a checkout its binary predates is debugging a difference
    that does not exist.

    It refuses in two cases, and both are refusals rather than warnings:

      * MISMATCH — the binary's `gui build <sha>` stamp is not the checkout's HEAD.
      * DIRTY — the stamp ends in `-dirty`, so the SHA does not describe what is running. There is
        no exact comparison to make, so there is no safe way to continue, and `-Force` is
        deliberately NOT offered for it.

    WHY A PROCEDURE CANNOT REPLACE IT: `crates/gui/build.rs` records that the running gui.exe has
    been stale six times, and on the sixth the runbook telling you to check the file's mtime named
    the wrong build too. A value compiled into the binary can close that; a habit cannot.

.PARAMETER Checkout
    The Windows checkout of this repository. DEFAULTS to the repository this script is in, so the
    script works unmoved from `scripts/` and would work equally at the checkout root.

.PARAMETER Exe
    The gui.exe to run. DEFAULTS to `<checkout>\.bin\gui.exe` — the drop directory, which is
    gitignored precisely so it cannot make the checkout read dirty. A binary already inside the
    checkout is run WHERE IT LIES; one from outside is copied to the drop first, so there is never
    a second candidate binary either.

.PARAMETER Port
    The daemon port. Defaults to 7451, which is also `protocol::DEFAULT_PORT` — so a bare `simd`
    with no argument listens on exactly this port.

.PARAMETER SkipFetch
    Do not fetch/pull the checkout first. The SHA check still runs.

.PARAMETER GuiArgs
    Anything else is forwarded to gui.exe unchanged, AFTER the port and `--assets`. Put `--` first
    when a flag might be mistaken for one of this script's own parameters; that form always works.

.EXAMPLE
    ./launch-gui.ps1
    # checkout = this repo, exe = .bin\gui.exe, assets = <checkout>\assets

.EXAMPLE
    ./launch-gui.ps1 -- --perf-log run.csv
    # the same launch, plus a frame log; read it afterwards with
    #   python3 scripts/bench/perf_summary.py run.csv

.EXAMPLE
    ./launch-gui.ps1 -Exe D:\drop\gui.exe -Port 7451

.NOTES
    WALKED ON WINDOWS 2026-09-07. Written in a Linux devpod with no Windows and no display, so it
    shipped unrun — and was then exercised at the seat: the happy path, MISMATCH (on a genuinely
    stale binary, not a staged one), `unrecognised stamp`, the diverged-checkout refusal, and the
    `-dirty` refusal. A check that has never been seen to refuse is a habit rather than a guard;
    these have now been seen. Transcripts are in the story's Verification section.
#>

[CmdletBinding()]
param(
    [string]$Checkout,
    [string]$Exe,
    # 7451, matching `protocol::DEFAULT_PORT` -- the constant was moved 7373 -> 7451 in this same
    # commit so the two cannot drift. A bare `simd` therefore lands on exactly this port.
    [int]$Port = 7451,
    [switch]$SkipFetch,
    # Everything else goes straight to gui.exe. This exists so that wanting a flag is never a
    # reason to bypass the SHA check -- a hand-run gui.exe is exactly the case where a stale binary
    # goes unnoticed, which is the failure this script was written to close.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$GuiArgs
)

$ErrorActionPreference = 'Stop'

function Fail($message) {
    Write-Host "launch-gui: $message" -ForegroundColor Red
    exit 1
}

# --- Defaults, derived from where this script lives -------------------------------------------
# Asked of GIT rather than assumed from the folder layout, so the script works unmoved from
# scripts/ AND from the checkout root, and keeps working if it is moved again.
if (-not $Checkout) {
    $Checkout = (& git -C $PSScriptRoot rev-parse --show-toplevel 2>$null)
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($Checkout)) {
        Fail "this script is not inside a git checkout ($PSScriptRoot), and -Checkout was not given"
    }
    $Checkout = $Checkout.Trim()
}
if (-not (Test-Path -LiteralPath $Checkout -PathType Container)) {
    Fail "checkout not found: $Checkout"
}
# Resolve once so every later comparison is between real paths, not a mix of forms.
$Checkout = (Resolve-Path -LiteralPath $Checkout).Path

$dropDir = Join-Path $Checkout '.bin'
if (-not $Exe) { $Exe = Join-Path $dropDir 'gui.exe' }

if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) {
    Fail @"
gui.exe not found: $Exe
Drop a build there (it is gitignored), or pass -Exe. Build it with --release: a debug gui.exe is
about 1.8 GB.
"@
}
$Exe = (Resolve-Path -LiteralPath $Exe).Path

$assets = Join-Path $Checkout 'assets'
if (-not (Test-Path -LiteralPath $assets -PathType Container)) {
    Fail "the checkout has no assets/ directory: $assets"
}

# --- Bring the checkout up to date ------------------------------------------------------------
if (-not $SkipFetch) {
    Write-Host "launch-gui: fetching $Checkout"
    & git -C $Checkout fetch --prune
    if ($LASTEXITCODE -ne 0) { Fail 'git fetch failed' }
    & git -C $Checkout pull --ff-only
    if ($LASTEXITCODE -ne 0) { Fail 'git pull --ff-only failed; the checkout has diverged' }
}

# The checkout's HEAD, short, to compare against the stamp. `build.rs` uses `git rev-parse --short`,
# so this must too, or the comparison is between two different lengths of the same commit.
$head = (& git -C $Checkout rev-parse --short HEAD)
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($head)) {
    Fail 'cannot read the checkout HEAD'
}
$head = $head.Trim()

# The checkout's OWN cleanliness, REPORTED and not refused. Local edits are normal here -- editing
# an asset and watching it reload is the whole point of `--assets` -- so refusing would fight the
# feature. But "gui.exe and checkout are both <sha>" is then a FALSE statement about the tree being
# served, and a launcher whose headline claim can be quietly wrong is the thing this script exists
# to replace. So the claim is made accurate instead.
$dirtyFiles = @(& git -C $Checkout status --porcelain)
$checkoutState = if ($dirtyFiles.Count -gt 0) {
    "$head (+$($dirtyFiles.Count) local change(s) -- the served tree is NOT exactly $head)"
} else {
    $head
}

# --- Ask the BINARY what it is ----------------------------------------------------------------
# Not the filename, not the mtime. `--version` prints `gui build <sha>` and exits without
# connecting to a daemon or opening a window.
# Captured in full BEFORE reading $LASTEXITCODE. Piping straight into `Select-Object -First 1`
# can stop the pipeline early, and the exit code then reflects whatever ran last rather than the
# binary -- which would make the one check whose job is trustworthiness read a stale status.
$stampOutput = & $Exe --version 2>&1
$stampExit = $LASTEXITCODE
$stampLine = ($stampOutput | Select-Object -First 1)
# A binary that SUPPORTS --version always exits 0, so any non-zero status here means it does not
# support the flag: it treated `--version` as the positional port argument and rejected it
# (`Error: invalid port`). That is the diagnosis, and it belongs on THIS branch rather than on the
# one below -- the message used to live on the `-notmatch` path, which needs exit 0, so an old
# binary produced a bare "failed (1)" while the sentence explaining it sat unreachable.
#
# NOTE the chicken-and-egg this closes: the SHA comparison is what reports a stale binary, and a
# binary too old to answer --version cannot reach that comparison. So a build predating the flag
# has to be named HERE or it is reported as a mystery.
if ($stampExit -ne 0) {
    Fail @"
this gui.exe does not support --version (exit $stampExit): $stampLine
It predates the flag, so it cannot be identified and cannot be checked against the checkout.
`--version` landed with story 10.5b: build from a branch that contains it (10-5b-the-art-iteration-loop
at time of writing), not from main.
"@
}
if ($stampLine -notmatch '^gui build (?<sha>\S+)$') {
    Fail "unrecognised stamp from gui.exe --version: '$stampLine' (expected 'gui build <sha>')"
}
$stamp = $Matches['sha']

if ($stamp -eq 'unknown') {
    Fail 'the binary carries no build stamp (built outside a git tree); there is nothing to verify'
}
if ($stamp -match '-dirty$' -or $stamp -match '-unknown-dirtiness$') {
    Fail @"
the binary is stamped '$stamp' — it was built from an UNCOMMITTED tree, so its SHA does not
describe what is running and no exact comparison exists. Commit the source and rebuild.
"@
}
if ($stamp -ne $head) {
    Fail @"
MISMATCH — the binary and the checkout are different commits.
    gui.exe   $stamp
    checkout  $head
Rebuild gui.exe at $head, or check the tree out at $stamp. Launching would compare a dwarf
against assets its binary never saw.
"@
}

Write-Host "launch-gui: verified — gui.exe $stamp, checkout $checkoutState" -ForegroundColor Green
if ($dirtyFiles.Count -gt 0) {
    Write-Host "launch-gui: the checkout has uncommitted changes; --assets serves them, not $head" -ForegroundColor Yellow
    $dirtyFiles | Select-Object -First 5 | ForEach-Object { Write-Host "    $_" -ForegroundColor Yellow }
    if ($dirtyFiles.Count -gt 5) { Write-Host "    ... and $($dirtyFiles.Count - 5) more" -ForegroundColor Yellow }
}

# --- Run it -----------------------------------------------------------------------------------
# A binary already inside the checkout is run WHERE IT LIES. Copying it elsewhere would create the
# second candidate binary this script exists to prevent, and the copy would be the one nobody can
# trace back to a commit.
# Compared with a trailing separator. A bare StartsWith would treat `D:\frostvein\...` as living
# inside a checkout at `D:\frost`, and then run a binary from a different tree in place.
$checkoutPrefix = $Checkout.TrimEnd([System.IO.Path]::DirectorySeparatorChar) +
    [System.IO.Path]::DirectorySeparatorChar
if ($Exe.StartsWith($checkoutPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    $target = $Exe
} else {
    if (-not (Test-Path -LiteralPath $dropDir -PathType Container)) {
        New-Item -ItemType Directory -Path $dropDir | Out-Null
    }
    $target = Join-Path $dropDir 'gui.exe'
    # Copied only AFTER the check passes, so a refused launch never leaves a newer binary behind
    # claiming to belong to this checkout.
    Copy-Item -LiteralPath $Exe -Destination $target -Force
    Write-Host "launch-gui: copied to $target"
}

# ORDER IS DELIBERATE: port, then --assets, then whatever was forwarded. A forwarded `--assets`
# therefore lands LAST, and `gui`'s parser takes the last occurrence -- so the launcher's default is
# overridable rather than a wall.
$argv = @([string]$Port, '--assets', $assets) + $GuiArgs
if ($GuiArgs) { Write-Host "launch-gui: forwarding $($GuiArgs -join ' ')" }
Write-Host "launch-gui: starting on port $Port with --assets $assets"
& $target @argv
exit $LASTEXITCODE
