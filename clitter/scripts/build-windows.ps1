# Clitter Windows Build Script
# Usage: .\scripts\build-windows.ps1

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "=== Clitter Windows Build ===" -ForegroundColor Cyan

# Check prerequisites
Write-Host "`nChecking prerequisites..." -ForegroundColor Yellow

# Rust
if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found. Installing..." -ForegroundColor Red
    winget install Rustlang.Rustup
    Write-Host "Please restart terminal and run this script again." -ForegroundColor Yellow
    exit 1
}
Write-Host "  Rust: $(rustc --version)" -ForegroundColor Green

# Node.js
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Node.js not found. Installing..." -ForegroundColor Red
    winget install OpenJS.NodeJS.LTS
    Write-Host "Please restart terminal and run this script again." -ForegroundColor Yellow
    exit 1
}
Write-Host "  Node.js: $(node --version)" -ForegroundColor Green

# npm
if (!(Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host "npm not found." -ForegroundColor Red
    exit 1
}
Write-Host "  npm: $(npm --version)" -ForegroundColor Green

# Move to clitter directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectDir = Split-Path -Parent $scriptDir
Set-Location $projectDir

Write-Host "`nInstalling dependencies..." -ForegroundColor Yellow
npm install

Write-Host "`nBuilding Tauri app..." -ForegroundColor Yellow
npm run tauri build

Write-Host "`n=== Build Complete ===" -ForegroundColor Cyan
Write-Host "`nOutput files:" -ForegroundColor Yellow
Write-Host "  Executable: src-tauri\target\release\clitter.exe"
Write-Host "  MSI:        src-tauri\target\release\bundle\msi\"
Write-Host "  NSIS:       src-tauri\target\release\bundle\nsis\"

# Open output folder
$releaseDir = Join-Path $projectDir "src-tauri\target\release"
if (Test-Path $releaseDir) {
    Write-Host "`nOpening release folder..." -ForegroundColor Yellow
    explorer $releaseDir
}
