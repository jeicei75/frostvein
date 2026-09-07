<#
.SYNOPSIS
    Fetch the checkout, copy the fresh gui.exe, and launch it against that checkout's assets —
    refusing to launch if the two are not the same commit.

.DESCRIPTION
    Story 10.5b Task 4; closes issue #46 (M2-7, "automate the gui.exe rebuild-and-copy — the stamp
    half is done").

    THE CHECK IS THE POINT, NOT THE COPYING. A launcher that only copies is a convenience. This one
    exists because `--assets` introduces a SECOND candidate asset tree, and two candidate trees is
    the stale-artifact shape this project keeps paying for: the binary is built in a devpod and
    copied here, the checkout is fetched separately, and nothing but this script makes them one act.
    A session that reads a dwarf from a checkout its binary predates is debugging a difference that
    does not exist.

    It refuses in two cases, and both are refusals rather than warnings:

      * MISMATCH — the binary's `gui build <sha>` stamp is not the checkout's HEAD. The two came
        from different commits and the run would prove nothing.
      * DIRTY — the stamp ends in `-dirty`, meaning the build had uncommitted changes and the SHA
        does NOT describe what is running. There is no exact comparison to make, so there is no
        safe way to continue; `-Force` is deliberately NOT offered for this.

    WHY A PROCEDURE CANNOT REPLACE IT: `crates/gui/build.rs` records that the running gui.exe has
    been stale six times, and on the sixth the runbook telling you to check the file's mtime named
    the wrong build too. A value compiled into the binary can close that; a habit cannot.

.PARAMETER Checkout
    The Windows checkout of this repository. Its `assets/` directory is what `--assets` is pointed
    at, and its HEAD is what the binary's stamp must match.

.PARAMETER Exe
    The freshly built gui.exe to copy in — normally the artifact just pulled out of the devpod.

.PARAMETER Port
    The daemon port to connect to. Defaults to the protocol default.

.PARAMETER SkipFetch
    Do not `git fetch`/`git pull` the checkout first. The SHA check still runs.

.EXAMPLE
    ./launch-gui.ps1 -Checkout D:\frostvein -Exe D:\drop\gui.exe

.NOTES
    UNRUN ON WINDOWS AT AUTHORING TIME. This script was written in a Linux devpod that has no
    Windows and no display, so it has never been executed. The RED that must be observed before its
    green is trusted, and the exact expected output, are in the story's Verification section — run
    them at the seat before relying on the check.
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Checkout,
    [Parameter(Mandatory = $true)][string]$Exe,
    [int]$Port = 7878,
    [switch]$SkipFetch
)

$ErrorActionPreference = 'Stop'

function Fail($message) {
    Write-Host "launch-gui: $message" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path -LiteralPath $Checkout -PathType Container)) {
    Fail "checkout not found: $Checkout"
}
if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) {
    Fail "gui.exe not found: $Exe"
}

$assets = Join-Path $Checkout 'assets'
if (-not (Test-Path -LiteralPath $assets -PathType Container)) {
    Fail "the checkout has no assets/ directory: $assets"
}

if (-not $SkipFetch) {
    Write-Host "launch-gui: fetching $Checkout"
    & git -C $Checkout fetch --prune
    if ($LASTEXITCODE -ne 0) { Fail 'git fetch failed' }
    & git -C $Checkout pull --ff-only
    if ($LASTEXITCODE -ne 0) { Fail 'git pull --ff-only failed; the checkout has diverged' }
}

# The checkout's HEAD, short, to compare against the stamp. `build.rs` uses `git rev-parse --short`,
# so this must too or the comparison is between two different lengths of the same commit.
$head = (& git -C $Checkout rev-parse --short HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($head)) {
    Fail 'cannot read the checkout HEAD'
}

# Ask the BINARY what it is, rather than inferring it from a filename or an mtime. `--version`
# prints `gui build <sha>` and exits without connecting to a daemon or opening a window.
$stampLine = (& $Exe --version 2>&1 | Select-Object -First 1)
if ($LASTEXITCODE -ne 0) { Fail "gui.exe --version failed: $stampLine" }
if ($stampLine -notmatch '^gui build (?<sha>\S+)$') {
    Fail "unrecognised stamp from gui.exe --version: '$stampLine'"
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

# Copy in only after the check passes, so a refused launch never leaves a newer binary behind
# claiming to belong to this checkout.
$target = Join-Path $Checkout 'gui.exe'
Copy-Item -LiteralPath $Exe -Destination $target -Force
Write-Host "launch-gui: copied to $target"

$assetsFull = (Resolve-Path -LiteralPath $assets).Path
Write-Host "launch-gui: starting on port $Port with --assets $assetsFull"
& $target $Port --assets $assetsFull
exit $LASTEXITCODE
