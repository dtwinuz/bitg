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
  url64bit       = 'https://github.com/dtwinuz/bitg/releases/download/v0.4.0/bitg-0.4.0-x64.msi'
  checksum64     = '5bddcdafe8e780fee3d9c869861b04b61a3e7cf6c637976834c659c5695c7171'
  checksumType64 = 'sha256'
  softwareName   = 'bitg'
  silentArgs     = "/qn /norestart /l*v `"$log`" $($props -join ' ')"
  validExitCodes = @(0, 3010, 1641)
}
Install-ChocolateyPackage @packageArgs
