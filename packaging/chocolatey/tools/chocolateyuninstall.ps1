$ErrorActionPreference = 'Stop'

# ProductCode har build'da yangi: o'rnatilgan yozuv nomi bo'yicha topiladi (Publisher ham tekshiriladi).
$keys = @(Get-UninstallRegistryKey -SoftwareName 'bitg' | Where-Object { $_.Publisher -eq 'dtwinuz' })
if ($keys.Count -eq 0) {
  Write-Warning "bitg o'rnatilmagan ko'rinadi: o'chiradigan narsa yo'q."
  return
}
foreach ($k in $keys) {
  Uninstall-ChocolateyPackage -PackageName $env:ChocolateyPackageName -FileType 'msi' `
    -SilentArgs "$($k.PSChildName) /qn /norestart" -ValidExitCodes @(0, 3010, 1605, 1614, 1641) -File ''
}
