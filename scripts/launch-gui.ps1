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
    The daemon port. Defaults to the protocol default.

.PARAMETER SkipFetch
    Do not fetch/pull the checkout first. The SHA check still runs.

.EXAMPLE
    ./launch-gui.ps1
    # checkout = this repo, exe = .bin\gui.exe, assets = <checkout>\assets

.EXAMPLE
    ./launch-gui.ps1 -Exe D:\drop\gui.exe -Port 7878

.NOTES
    UNRUN ON WINDOWS AT AUTHORING TIME. Written in a Linux devpod with no Windows and no display,
    so it has never been executed. The RED to walk before its green is trusted, and the exact
    expected output, are in the story's Verification section. A check that has never been seen to
    refuse is a habit, not a guard.
#>

[CmdletBinding()]
param(
    [string]$Checkout,
    [string]$Exe,
    [int]$Port = 7878,
    [switch]$SkipFetch
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

# --- Ask the BINARY what it is ----------------------------------------------------------------
# Not the filename, not the mtime. `--version` prints `gui build <sha>` and exits without
# connecting to a daemon or opening a window.
# Captured in full BEFORE reading $LASTEXITCODE. Piping straight into `Select-Object -First 1`
# can stop the pipeline early, and the exit code then reflects whatever ran last rather than the
# binary -- which would make the one check whose job is trustworthiness read a stale status.
$stampOutput = & $Exe --version 2>&1
$stampExit = $LASTEXITCODE
$stampLine = ($stampOutput | Select-Object -First 1)
if ($stampExit -ne 0) { Fail "gui.exe --version failed ($stampExit): $stampLine" }
if ($stampLine -notmatch '^gui build (?<sha>\S+)$') {
    Fail @"
unrecognised stamp from gui.exe --version: '$stampLine'
A build predating story 10.5b has no --version flag at all, and will have tried to start the
client instead. Drop a newer binary.
"@
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

Write-Host "launch-gui: verified — gui.exe and checkout are both $head" -ForegroundColor Green

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

Write-Host "launch-gui: starting on port $Port with --assets $assets"
& $target $Port --assets $assets
exit $LASTEXITCODE
