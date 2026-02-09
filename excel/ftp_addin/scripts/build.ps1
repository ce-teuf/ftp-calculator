# build.ps1 — Build the FTP Excel Add-In (Rust DLL + C# Excel-DNA)
#
# Usage:  powershell -ExecutionPolicy Bypass -File build.ps1
#         or from the repo root:  .\excel\ftp_addin\scripts\build.ps1

param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"

$repoRoot  = Resolve-Path "$PSScriptRoot\..\..\..\"
$rustProj  = "ftp_core_bindings_c"
$csProj    = "$PSScriptRoot\..\ftp_addin.csproj"

Write-Host "=== Step 1: Build Rust cdylib ($Configuration) ===" -ForegroundColor Cyan

$cargoProfile = if ($Configuration -eq "Release") { "--release" } else { "" }
$cmd = "cargo build -p $rustProj $cargoProfile"
Write-Host $cmd
Push-Location $repoRoot
Invoke-Expression $cmd
if ($LASTEXITCODE -ne 0) { throw "Rust build failed" }
Pop-Location

# Determine Rust output directory
$rustOutDir = if ($Configuration -eq "Release") {
    Join-Path $repoRoot "target\release"
} else {
    Join-Path $repoRoot "target\debug"
}
$rustDll = Join-Path $rustOutDir "$rustProj.dll"
if (-not (Test-Path $rustDll)) { throw "Rust DLL not found: $rustDll" }
Write-Host "Rust DLL: $rustDll" -ForegroundColor Green

Write-Host ""
Write-Host "=== Step 2: Build C# Excel-DNA add-in ===" -ForegroundColor Cyan

dotnet build $csProj -c $Configuration
if ($LASTEXITCODE -ne 0) { throw "C# build failed" }

# Determine .NET output directory
$dotnetOutDir = Join-Path "$PSScriptRoot\.." "bin\$Configuration\net6.0-windows"
$dotnetOutDir = Resolve-Path $dotnetOutDir

Write-Host ""
Write-Host "=== Step 3: Copy Rust DLL alongside .xll ===" -ForegroundColor Cyan

$destDll = Join-Path $dotnetOutDir "$rustProj.dll"
Copy-Item $rustDll $destDll -Force
Write-Host "Copied $rustDll -> $destDll" -ForegroundColor Green

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Add-in output: $dotnetOutDir"
Write-Host "Open the .xll file in Excel to load the add-in."
