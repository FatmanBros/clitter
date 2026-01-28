# Clitter Windows Dev Script
# Usage: .\scripts\dev-windows.ps1

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectDir = Split-Path -Parent $scriptDir
Set-Location $projectDir

Write-Host "=== Clitter Dev Mode ===" -ForegroundColor Cyan

if (!(Test-Path "node_modules")) {
    Write-Host "Installing dependencies..." -ForegroundColor Yellow
    npm install
}

Write-Host "Starting dev server..." -ForegroundColor Yellow
npm run tauri:dev
