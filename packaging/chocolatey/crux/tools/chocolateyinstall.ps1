$ErrorActionPreference = 'Stop'
$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$version = '0.1.0'
$url = "https://github.com/Emran-goat/crux/releases/download/v$version/crux-x86_64-pc-windows-msvc.zip"
$checksum = 'PLACEHOLDER_SHA256'

$packageArgs = @{
  packageName   = 'crux'
  unzipLocation = $toolsDir
  url           = $url
  checksum      = $checksum
  checksumType  = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
