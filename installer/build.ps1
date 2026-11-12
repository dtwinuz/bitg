# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

<#
.SYNOPSIS
  bitg MSI'ni quradi: Rust binarlari (statik CRT) -> ixtiyoriy imzo -> WiX MSI -> MSI imzosi -> SHA256.

.EXAMPLE
  pwsh installer/build.ps1                       # versiya Cargo.toml'dan, imzosiz
  pwsh installer/build.ps1 -Version 0.3.0
  pwsh installer/build.ps1 -CertThumbprint ABC…  # sertifikat do'konidagi kalit bilan imzolash

  CI: BITG_SIGN_PFX (base64) va BITG_SIGN_PFX_PASSWORD bo'lsa — PFX bilan imzolanadi.

.OUTPUTS
  dist/bitg-<versiya>-x64.msi va dist/bitg-<versiya>-x64.msi.sha256
#>
[CmdletBinding()]
param(
    [string]$Version,
    [string]$CertThumbprint = $env:BITG_SIGN_THUMBPRINT,
    [string]$TimestampUrl = 'http://timestamp.digicert.com',
    [switch]$SkipCargo
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
Push-Location $root
try {
    if (-not $Version) {
        $m = Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
        $Version = $m.Matches[0].Groups[1].Value
    }
    if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Versiya X.Y.Z bo'lishi kerak (MSI talabi): $Version" }
    Write-Host "bitg $Version" -ForegroundColor Cyan

    if (-not $SkipCargo) {
        # MSVC majburiy: GNU toolchain'da WebView2Loader.dll alohida fayl bo'lib qoladi (o'rnatilgan ilova ochilmaydi)
        # va IME DLL boshqa CRT bilan quriladi. Standart toolchain GNU bo'lgan kompyuterda ham shu ishlatiladi.
        if (-not $env:RUSTUP_TOOLCHAIN) { $env:RUSTUP_TOOLCHAIN = 'stable-x86_64-pc-windows-msvc' }
        cargo build --release --locked -p bitg-tsf -p bitg-settings -p bitg-cli
        if ($LASTEXITCODE) { throw 'cargo build' }
        # 32-bit dasturlar uchun IME DLL (rustup target add i686-pc-windows-msvc).
        cargo build --release --locked -p bitg-tsf --target i686-pc-windows-msvc
        if ($LASTEXITCODE) { throw 'cargo build (i686)' }
    }
    $bin = Join-Path $root 'target\release'
    $bin32 = Join-Path $root 'target\i686-pc-windows-msvc\release'
    $binaries = @('bitg_tsf.dll', 'bitg-settings.exe', 'bitg.exe' | ForEach-Object { Join-Path $bin $_ }) +
        @(Join-Path $bin32 'bitg_tsf.dll')
    foreach ($b in $binaries) {
        if (Select-String -Path $b -Pattern 'VCRUNTIME140' -SimpleMatch -Quiet) {
            throw "$b VC++ runtime'ga bog'liq (.cargo/config.toml crt-static ishlamadi)"
        }
    }

    # --- imzo ---
    $pfx = $null
    if ($env:BITG_SIGN_PFX) {
        $pfx = Join-Path ([IO.Path]::GetTempPath()) "bitg-sign-$([guid]::NewGuid()).pfx"
        [IO.File]::WriteAllBytes($pfx, [Convert]::FromBase64String($env:BITG_SIGN_PFX))
    }
    $signtool = $null
    if ($CertThumbprint -or $pfx) {
        $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
            Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1 -ExpandProperty FullName
        if (-not $signtool) { throw 'signtool.exe topilmadi (Windows SDK kerak)' }
    }
    function Sign([string[]]$files) {
        if (-not $signtool) { return }
        $sa = @('sign', '/fd', 'SHA256', '/tr', $TimestampUrl, '/td', 'SHA256', '/d', 'bitg', '/du', 'https://bitg.uz')
        if ($pfx) { $sa += @('/f', $pfx, '/p', $env:BITG_SIGN_PFX_PASSWORD) } else { $sa += @('/sha1', $CertThumbprint) }
        & $signtool @sa @files
        if ($LASTEXITCODE) { throw 'signtool sign' }
    }
    try {
        Sign $binaries

        # --- MSI ---
        dotnet build installer/bitg.wixproj -c Release -t:Rebuild -nologo -v:minimal `
            "-p:Version=$Version" "-p:BinDir=$bin\" "-p:BinDir32=$bin32\"
        if ($LASTEXITCODE) { throw 'WiX build' }
        $dist = Join-Path $root 'dist'
        New-Item -ItemType Directory -Force $dist | Out-Null
        $msi = Join-Path $dist "bitg-$Version-x64.msi"
        Copy-Item installer\bin\x64\Release\en-US\bitg.msi $msi -Force
        Sign @($msi)
    }
    finally {
        if ($pfx) { Remove-Item $pfx -Force -ErrorAction SilentlyContinue }
    }

    $hash = (Get-FileHash $msi -Algorithm SHA256).Hash.ToLower()
    "$hash  $(Split-Path $msi -Leaf)" | Set-Content -Encoding ascii "$msi.sha256"
    $signed = if ($signtool) { 'imzolangan' } else { 'IMZOSIZ' }
    Write-Host "$msi ($signed)`nSHA256 $hash" -ForegroundColor Green
}
finally {
    Pop-Location
}
