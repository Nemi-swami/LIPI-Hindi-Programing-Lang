# LIPI one-command installer for Windows (PowerShell).
#   irm https://raw.githubusercontent.com/<you>/lipi-lang/main/install.ps1 | iex
#
# Clones the repo, builds with Cargo (release), and installs lipi.exe to
# %LOCALAPPDATA%\LIPI\bin (override with $env:LIPI_PREFIX). Requires: git, cargo.
$ErrorActionPreference = 'Stop'

$repo   = if ($env:LIPI_REPO)   { $env:LIPI_REPO }   else { 'https://github.com/naraxcel/lipi-lang.git' }
$prefix = if ($env:LIPI_PREFIX) { $env:LIPI_PREFIX } else { Join-Path $env:LOCALAPPDATA 'LIPI\bin' }
$work   = Join-Path $env:TEMP ("lipi-build-" + [guid]::NewGuid().ToString('N'))

Write-Host "LIPI installer" -ForegroundColor Cyan
Write-Host "=============="

foreach ($t in 'git','cargo') {
  if (-not (Get-Command $t -ErrorAction SilentlyContinue)) {
    throw "$t not found. Install Rust from https://rustup.rs and Git, then retry."
  }
}

Write-Host "-> cloning $repo"
git clone --depth 1 $repo $work | Out-Null

Write-Host "-> building (release) - this takes a minute"
Push-Location $work
cargo build --release
Pop-Location

New-Item -ItemType Directory -Force $prefix | Out-Null
Copy-Item -Force (Join-Path $work 'target\release\lipi.exe') (Join-Path $prefix 'lipi.exe')
Remove-Item -Recurse -Force $work

# Add to the user PATH if it's not already there.
$userPath = [Environment]::GetEnvironmentVariable('Path','User')
if ($userPath -notlike "*$prefix*") {
  [Environment]::SetEnvironmentVariable('Path', "$userPath;$prefix", 'User')
  Write-Host "  added $prefix to your PATH (restart the terminal to pick it up)"
}

Write-Host ""
Write-Host "installed to $prefix\lipi.exe" -ForegroundColor Green
Write-Host "  try:  lipi   (REPL)   or   'बताओ \"नमस्ते\"' > hi.swami ; lipi hi.swami"
