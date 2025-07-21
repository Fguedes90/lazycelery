$ErrorActionPreference = 'Stop'

# Input validation
$packageName = 'lazycelery'
if ($packageName -notmatch '^[a-zA-Z0-9\-]+$') {
    throw "Invalid package name: $packageName"
}

$version = '0.4.0'
if ($version -notmatch '^\d+\.\d+\.\d+(-[a-zA-Z0-9\-\.]+)?$') {
    throw "Invalid version format: $version"
}

$url64 = "https://github.com/Fguedes90/lazycelery/releases/download/v$version/lazycelery-windows-x86_64.zip"
if ($url64 -notmatch '^https://[a-zA-Z0-9\-\.]+/.*$') {
    throw "Invalid URL format: $url64"
}

# SHA256 checksum - should be updated during release process
$checksum64 = 'PLACEHOLDER_SHA256'
if ($checksum64 -eq 'PLACEHOLDER_SHA256') {
    Write-Warning "Using placeholder SHA256 checksum. This should be updated during release."
}

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Create shim for the executable with validation
$executableName = 'lazycelery.exe'
if ($executableName -notmatch '^[a-zA-Z0-9\-]+\.exe$') {
    throw "Invalid executable name: $executableName"
}

$exePath = Join-Path $toolsDir $executableName
if (Test-Path $exePath) {
    Install-BinFile -Name $packageName -Path $exePath
} else {
    throw "Executable not found at: $exePath"
}