$ErrorActionPreference = 'Stop'

$packageName = 'lazycelery'
$version = '0.2.0'
$url64 = "https://github.com/Fguedes90/lazycelery/releases/download/v$version/lazycelery-windows-x86_64.zip"
$checksum64 = 'PLACEHOLDER_SHA256'

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Create shim for the executable
$exePath = Join-Path $toolsDir 'lazycelery.exe'
Install-BinFile -Name 'lazycelery' -Path $exePath