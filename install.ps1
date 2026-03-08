# LazyCelery Install Script for Windows
# Usage:
#   irm https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.ps1 | iex
#   or
#   Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.ps1" -OutFile install.ps1; .\install.ps1

param(
    [string]$Version = "",
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\lazycelery",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$REPO = "Fguede s90/lazycelery"

function Write-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Error($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red }

function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
        return $response.tag_name -replace '^v', ''
    } catch {
        Write-Error "Could not fetch latest version: $_"
        exit 1
    }
}

function Get-TargetUrl {
    param([string]$Ver, [string]$Os, [string]$Arch)
    
    $filename = "lazycelery-$Os-$Arch.zip"
    return "https://github.com/$REPO/releases/download/v$Ver/$filename"
}

# Main
Write-Info "LazyCelery Installer for Windows"
Write-Info "Repository: https://github.com/$REPO"

if (-not $Version) {
    $Version = Get-LatestVersion
}

Write-Info "Target version: $Version"

# Detect architecture
$Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
$Os = "windows"

$Url = Get-TargetUrl -Ver $Version -Os $Os -Arch $Arch
Write-Info "Downloading from: $Url"

# Download
$TempDir = [System.IO.Path]::GetTempPath()
$ZipPath = Join-Path $TempDir "lazycelery-$([System.Guid]::NewGuid().ToString()).zip"

try {
    Write-Info "Downloading..."
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
    
    # Extract
    Write-Info "Extracting..."
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force
    
    # Find binary
    $Binary = Get-ChildItem -Path $TempDir -Filter "lazycelery*.exe" -Recurse | Select-Object -First 1
    
    if (-not $Binary) {
        Write-Error "Could not find lazycelery.exe in archive"
        exit 1
    }
    
    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # Check if binary exists
    $TargetPath = Join-Path $InstallDir "lazycelery.exe"
    if ((Test-Path $TargetPath) -and -not $Force) {
        Write-Warn "Binary already exists at $TargetPath"
        Write-Warn "Use -Force to overwrite"
        exit 1
    }
    
    # Install
    Copy-Item $Binary.FullName -Destination $TargetPath -Force
    Write-Info "Installed to $TargetPath"
    
    # Add to PATH if not already
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        $newPath = "$currentPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Info "Added $InstallDir to PATH"
        Write-Info "You may need to restart your terminal for changes to take effect"
    }
    
    # Verify
    try {
        & $TargetPath --version
        Write-Info "Verification passed!"
    } catch {
        Write-Warn "Verification failed, but binary was installed"
    }
    
    Write-Info "Installation complete!"
    Write-Info "Run 'lazycelery --help' to get started"
    
} finally {
    # Cleanup
    if (Test-Path $ZipPath) {
        Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue
    }
}
