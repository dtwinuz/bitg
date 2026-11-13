$ErrorActionPreference = 'Stop'

# MSI GitHub Release'dan olinadi va SHA-256 bilan tekshiriladi: fayl paket ichida emas.
$pp = Get-PackageParameters
$props = @()
if ($pp['NoAutostart']) { $props += 'AUTOSTART=0' }
if ($pp['NoKeyboard'])  { $props += 'ADDKEYBOARD=0' }
$log = Join-Path $env:TEMP "bitg-$env:ChocolateyPackageVersion-install.log"

$packageArgs = @{
  packageName    = $env:ChocolateyPackageName
  fileType       = 'msi'
  url64bit       = 'https://github.com/dtwinuz/bitg/releases/download/v0.3.1/bitg-0.3.1-x64.msi'
  checksum64     = '6cf4f743ec90392f1cf566e20de8c55d8505d249d0929de9ef3bdcd501c86236'
  checksumType64 = 'sha256'
  softwareName   = 'bitg'
  silentArgs     = "/qn /norestart /l*v `"$log`" $($props -join ' ')"
  validExitCodes = @(0, 3010, 1641)
}
Install-ChocolateyPackage @packageArgs
