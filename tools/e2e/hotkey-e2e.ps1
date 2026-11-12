# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

<#
.SYNOPSIS
  "Belgilangan matnni o'girish" tugmasining haqiqiy Windows sinovi: o'rnatilgan bitg-settings.exe (tray)
  ishga tushiriladi, WinForms TextBox'da matn belgilanadi, Ctrl+Shift+Y SendInput bilan bosiladi va natija
  hamda clipboard qaytarilgani tekshiriladi.

  FAQAT CI VM'da (GitHub Actions, windows-latest). Lokal kompyuterda ishga tushirma: SendInput tugmalari
  fokusdagi har qanday oynaga ketadi, clipboard ham ishlatiladi. Skript CI muhitini tekshiradi va aks holda
  to'xtaydi.

.EXAMPLE
  powershell -File tools/e2e/hotkey-e2e.ps1 -Out result-hotkey.txt
#>
param(
    [string]$Exe = 'C:\Program Files\bitg\bitg-settings.exe',
    [string]$Out = (Join-Path ([IO.Path]::GetTempPath()) 'bitg-e2e-hotkey.txt')
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw "Faqat CI VM uchun (SendInput). GITHUB_ACTIONS=true emas." }

Add-Type -AssemblyName System.Windows.Forms
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class Hk {
    [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] public struct U { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
    [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public U u; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    public static uint Scan(ushort scan, bool up) {
        var i = new INPUT { type = 1 };
        i.u.ki.wScan = scan;
        i.u.ki.dwFlags = 0x8u | (up ? 0x2u : 0u);
        return SendInput(1, new[] { i }, Marshal.SizeOf(typeof(INPUT)));
    }
}
'@
$CTRL = 0x1D; $SHIFT = 0x2A; $Y = 0x15; $U = 0x16

# Kutilgan matnlar (fayl ASCII qolsin deb belgi kodlari bilan): s, c va o' ning yangi va eski shakllari.
$sh = [string][char]0x015F; $ch = [string][char]0x00E7; $Oo = [string][char]0x00D6; $okina = [string][char]0x02BB

# --- Sozlamalar: tray ilovasi settings.ini ni sekundiga bir marta tekshiradi ---
$cfgDir = Join-Path $env:APPDATA 'bitg'
New-Item -ItemType Directory -Force $cfgDir | Out-Null
function Settings([hashtable]$s) {
    $v = [ordered]@{ enabled = 1; auto_convert = 0; keymap = 0; hotkey = 1; hotkey_keys = 'ctrl+shift+y'; hotkey_old_keys = '' }
    foreach ($k in $s.Keys) { $v[$k] = $s[$k] }
    $text = ($v.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join "`n"
    [IO.File]::WriteAllText((Join-Path $cfgDir 'settings.ini'), "$text`ndisabled_apps=`n")
}
Remove-Item (Join-Path $cfgDir 'exceptions.txt') -ErrorAction SilentlyContinue
Settings @{}

$log = New-Object System.Collections.Generic.List[string]
$hkLog = Join-Path $env:LOCALAPPDATA 'bitg\hotkey.log'
Remove-Item $hkLog -ErrorAction SilentlyContinue

# --- Tray ilovasi (MSI AUTOSTART=0 bilan o'rnatilgan, o'zi ishga tushmagan) ---
if (-not (Test-Path $Exe)) { throw "topilmadi: $Exe" }
$app = Start-Process -FilePath $Exe -ArgumentList '--tray' -PassThru
$t0 = Get-Date
while (-not ((Test-Path $hkLog) -and (Select-String -Path $hkLog -Pattern 'registered' -Quiet))) {
    if (((Get-Date) - $t0).TotalSeconds -gt 20) { break }
    Start-Sleep -Milliseconds 200
}
$log.Add("app pid=$($app.Id) registered=" + ((Test-Path $hkLog) -and (Select-String -Path $hkLog -Pattern 'registered' -Quiet)))

# --- Matn maydoni ---
$win = New-Object System.Windows.Forms.Form
$win.Text = 'bitg e2e hotkey'; $win.Width = 600; $win.Height = 160
$tb = New-Object System.Windows.Forms.TextBox
$tb.Dock = 'Fill'; $tb.Font = New-Object System.Drawing.Font('Segoe UI', 14)
$win.Controls.Add($tb)

$SENTINEL = 'bitg-e2e-clipboard'
function Focus {
    $win.Activate()
    [void][Hk]::SetForegroundWindow($win.Handle)
    $tb.Focus() | Out-Null
    $log.Add("foreground=" + ([Hk]::GetForegroundWindow() -eq $win.Handle))
}
function Prepare([string]$text, [int]$start, [int]$len) {
    [System.Windows.Forms.Clipboard]::SetText($SENTINEL)
    $tb.Text = $text
    $tb.Select($start, $len)
}
# Ctrl+Shift+<tugma>: modifikatorlar ushlab turiladi ($hold) — ilova ularni o'zi qo'yib yuborishi kerak.
function Press([int]$key) {
    [void][Hk]::Scan($CTRL, $false); [void][Hk]::Scan($SHIFT, $false)
    [void][Hk]::Scan($key, $false); [void][Hk]::Scan($key, $true)
}
function Release { [void][Hk]::Scan($SHIFT, $true); [void][Hk]::Scan($CTRL, $true) }
function Check([string]$name, [string]$want) {
    $got = $tb.Text
    if ($got -ceq $want) { $log.Add("PASS $name") } else { $log.Add("FAIL $name got=[$got] want=[$want]") }
}
function CheckClipboard([string]$name) {
    $got = [System.Windows.Forms.Clipboard]::GetText()
    if ($got -ceq $SENTINEL) { $log.Add("PASS ${name}: clipboard restored") } else { $log.Add("FAIL ${name}: clipboard=[$got]") }
}

# Har qadam 350 ms. $w — kutish (o'girish ~0.5 s, sozlama o'zgarishi ~1 s).
$w = { }
$steps = @(
    { Focus }, $w,
    # 1) Hammasi belgilangan, tugmalar hali ushlab turilgan
    { Prepare 'Toshkent shahri choyxonasi' 0 26 }, { Press $Y }, { Release }, $w, $w, $w,
    { Check 'select all' "To${sh}kent ${sh}ahri ${ch}oyxonasi" }, { CheckClipboard 'select all' },
    # 2) Qisman belgilash: faqat birinchi so'z, tugmalar darhol qo'yib yuborilgan
    { Prepare 'choy va shakar' 0 4 }, { Press $Y; Release }, $w, $w, $w,
    { Check 'partial selection' "${ch}oy va shakar" }, { CheckClipboard 'partial' },
    # 3) Hech narsa belgilanmagan: matn ham, clipboard ham o'zgarmaydi
    { Prepare 'shahar' 6 0 }, { Press $Y; Release }, $w, $w, $w,
    { Check 'no selection' 'shahar' }, { CheckClipboard 'no selection' },
    # 4) Sozlamada o'chirilgan: tugma ro'yxatdan chiqariladi
    { Settings @{ hotkey = 0 } }, $w, $w, $w, $w, $w,
    { Prepare 'shahar' 0 6 }, { Press $Y; Release }, $w, $w, $w,
    { Check 'hotkey off' 'shahar' },
    # 5) Teskari yo'nalish (hotkey_old_keys): yangi -> eski
    { Settings @{ hotkey_old_keys = 'ctrl+shift+u' } }, $w, $w, $w, $w, $w,
    { Prepare "${Oo}zbekiston" 0 10 }, { Press $U; Release }, $w, $w, $w,
    { Check 'reverse hotkey' "O${okina}zbekiston" }, { CheckClipboard 'reverse' }
)

$script:k = 0
$tick = {
    if ($script:k -lt $steps.Count) {
        try { & $steps[$script:k] } catch { $log.Add("ERROR step $($script:k): $_") }
        $script:k++
    } else {
        $script:timer.Stop()
        $win.Close()
    }
}
$win.Add_Shown({
        $script:timer = New-Object System.Windows.Forms.Timer
        $script:timer.Interval = 350
        $script:timer.Add_Tick($tick)
        $script:timer.Start()
    })
[System.Windows.Forms.Application]::Run($win)

if (Test-Path $hkLog) { foreach ($l in Get-Content $hkLog) { $log.Add("log: $l") } }
Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue
Settings @{}
[IO.File]::WriteAllLines($Out, $log, (New-Object Text.UTF8Encoding $false))
Get-Content $Out -Encoding UTF8 | ForEach-Object { Write-Host $_ }
if (Select-String -Path $Out -Pattern '^(FAIL|ERROR)' -Quiet) { exit 1 }
