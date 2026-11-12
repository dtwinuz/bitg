# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

﻿<#
.SYNOPSIS
  bitg IME'ning haqiqiy Windows sinovi: tugmalar SendInput bilan bosiladi, natija matn maydonidan o'qiladi.

  FAQAT CI VM'da (GitHub Actions, windows-latest). Lokal kompyuterda ishga tushirma: SendInput tugmalari
  fokusdagi har qanday oynaga ketadi. Skript CI muhitini tekshiradi va aks holda to'xtaydi.

.PARAMETER Ui
  wpf      — WPF TextBox (TSF'ni o'zi qo'llaydi)
  winforms — WinForms TextBox (Win32 EDIT, IMM32/CUAS orqali)

.EXAMPLE
  powershell -File tools/e2e/tsf-e2e.ps1 -Ui wpf -Out result-wpf.txt
  C:\Windows\SysWOW64\WindowsPowerShell\v1.0\powershell.exe -File tools/e2e/tsf-e2e.ps1 -Ui wpf   # 32-bit DLL
#>
param(
    [ValidateSet('wpf', 'winforms')][string]$Ui = 'wpf',
    [string]$Out = (Join-Path ([IO.Path]::GetTempPath()) "bitg-e2e-$Ui.txt")
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw "Faqat CI VM uchun (SendInput). GITHUB_ACTIONS=true emas." }

Add-Type -AssemblyName PresentationFramework, System.Windows.Forms
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
[ComImport, Guid("71c6e74c-0f28-11d8-a82a-00065b84435c"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface ITfInputProcessorProfileMgr {
    [PreserveSig] int ActivateProfile(uint type, ushort langid, ref Guid clsid, ref Guid profile, IntPtr hkl, uint flags);
}
public static class E2e {
    [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] public struct U { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
    [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public U u; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();

    public static int ActivateBitg() {
        var t = Type.GetTypeFromCLSID(new Guid("33C53A50-F456-4884-B049-85FD643ECFED"));
        var pm = (ITfInputProcessorProfileMgr)Activator.CreateInstance(t);
        var clsid = new Guid("e37d4f70-f82a-435e-af8b-82e264391062");
        var prof = new Guid("bed48fc8-8690-4712-bbe0-1fd3284ce7b7");
        return pm.ActivateProfile(1, 0x0443, ref clsid, ref prof, IntPtr.Zero, 0x10000000);
    }
    // Skan-kod bilan (IME lParam'da skan-kodni ko'radi). ext — kengaytirilgan (o'ng Alt).
    public static uint Scan(ushort scan, bool up, bool ext) {
        var i = new INPUT { type = 1 };
        i.u.ki.wScan = scan;
        i.u.ki.dwFlags = 0x8u | (up ? 0x2u : 0u) | (ext ? 0x1u : 0u);
        return SendInput(1, new[] { i }, Marshal.SizeOf(typeof(INPUT)));
    }
}
'@

# --- US tartibi: belgi -> skan-kod (katta harf = Shift) ---
$scan = @{}
$digits = @(0x29, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D)
$i = 0; foreach ($c in '`1234567890-='.ToCharArray()) { $scan[[string]$c] = $digits[$i]; $i++ }
$i = 0; foreach ($c in 'qwertyuiop[]'.ToCharArray()) { $scan[[string]$c] = 0x10 + $i; $i++ }
$i = 0; foreach ($c in "asdfghjkl;'".ToCharArray()) { $scan[[string]$c] = 0x1E + $i; $i++ }
$i = 0; foreach ($c in 'zxcvbnm,./'.ToCharArray()) { $scan[[string]$c] = 0x2C + $i; $i++ }
$scan[' '] = 0x39
$shifted = @{ '{' = '['; '}' = ']'; ':' = ';'; '"' = "'" }
$SHIFT = 0x2A; $RALT = 0x38; $BACK = 0x0E

function Tap([int]$s) { [void][E2e]::Scan($s, $false, $false); [void][E2e]::Scan($s, $true, $false) }
function TypeKeys([string]$keys) {
    foreach ($ch in $keys.ToCharArray()) {
        $c = [string]$ch
        $up = ($c -cmatch '[A-Z]') -or $shifted.ContainsKey($c)
        $base = if ($shifted.ContainsKey($c)) { $shifted[$c] } else { $c.ToLower() }
        $s = $scan[$base]
        if ($null -eq $s) { throw "skan-kod yo'q: $c" }
        if ($up) { [void][E2e]::Scan($SHIFT, $false, $false) }
        Tap $s
        if ($up) { [void][E2e]::Scan($SHIFT, $true, $false) }
    }
}
function AltGr([string]$key) {
    [void][E2e]::Scan($RALT, $false, $true); Tap $scan[$key]; [void][E2e]::Scan($RALT, $true, $true)
}

# --- Sozlamalar: IME %APPDATA%\bitg ni sekundiga bir marta tekshiradi ---
$cfgDir = Join-Path $env:APPDATA 'bitg'
New-Item -ItemType Directory -Force $cfgDir | Out-Null
function Settings([hashtable]$s) {
    # undo_remember standart holatda 0; e2e istisnoga yozish yo'lini ham tekshirishi uchun 1.
    $v = @{ enabled = 1; auto_convert = 1; keymap = 0; layout = 'tilmoch'; undo_backspace = 1; undo_remember = 1; fallback = 1 }
    foreach ($k in $s.Keys) { $v[$k] = $s[$k] }
    $text = ($v.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join "`n"
    [IO.File]::WriteAllText((Join-Path $cfgDir 'settings.ini'), "$text`ndisabled_apps=`n")
}
Remove-Item (Join-Path $cfgDir 'exceptions.txt') -ErrorAction SilentlyContinue
Settings @{}

# --- Matn maydoni ---
if ($Ui -eq 'wpf') {
    $win = New-Object System.Windows.Window
    $win.Title = "bitg e2e $Ui"; $win.Width = 600; $win.Height = 160
    $tb = New-Object System.Windows.Controls.TextBox
    $tb.FontSize = 20
    $win.Content = $tb
} else {
    $win = New-Object System.Windows.Forms.Form
    $win.Text = "bitg e2e $Ui"; $win.Width = 600; $win.Height = 160
    $tb = New-Object System.Windows.Forms.TextBox
    $tb.Dock = 'Fill'; $tb.Font = New-Object System.Drawing.Font('Segoe UI', 14)
    $win.Controls.Add($tb)
}
$log = New-Object System.Collections.Generic.List[string]
$bits = if ([Environment]::Is64BitProcess) { 'x64' } else { 'x86' }
$log.Add("ui=$Ui process=$bits")
function ClearBox { $tb.Clear() }
function Check([string]$name, [string]$want) {
    $got = $tb.Text
    if ($got -ceq $want) { $log.Add("PASS $name") } else { $log.Add("FAIL $name got=[$got] want=[$want]") }
}
function Focus {
    if ($Ui -eq 'wpf') {
        $win.Activate() | Out-Null
        $h = (New-Object System.Windows.Interop.WindowInteropHelper $win).Handle
    } else {
        $win.Activate()
        $h = $win.Handle
    }
    [void][E2e]::SetForegroundWindow($h)
    $tb.Focus() | Out-Null
    $log.Add("foreground=" + ([E2e]::GetForegroundWindow() -eq $h))
}

# Har qadam 350 ms. $w — kutish; sozlama o'zgargach 4 ta (IME uni >1 s da qayta o'qiydi),
# keyin "a" + bo'shliq — IME yangi sozlamani shu tugmada oladi (AltGr ham shunda), bo'shliq so'zni yopadi:
# ClearBox matnni tugmasiz tozalaydi va zaxira yo'l (Win32 EDIT) buni ko'rmaydi.
$w = { }
$steps = @(
    { Focus },
    { $log.Add("ActivateProfile hr=0x{0:x}" -f [E2e]::ActivateBitg()) },
    { Focus }, $w,
    # 1) Avto o'girish: probeldan keyin so'z yangi alifboga
    { ClearBox }, { TypeKeys 'shahar ' }, $w, { Check 'auto: shahar' 'şahar ' },
    # 2) Darhol Backspace: asl so'z qaytadi va (undo_remember=1) istisnoga qo'shiladi
    { Tap $BACK }, $w, { Check 'undo: backspace restores' 'shahar' },
    {
        $ex = Join-Path $cfgDir 'exceptions.txt'
        $has = (Test-Path $ex) -and ((Get-Content $ex -Encoding UTF8) -contains 'shahar')
        $log.Add($(if ($has) { 'PASS undo: added to exceptions.txt' } else { 'FAIL undo: exceptions.txt has no shahar' }))
    },
    { TypeKeys ' ' }, $w, { Check 'undo: exception kept on next space' 'shahar ' },
    # 3) Inglizcha so'z o'zgarmaydi
    # Tez yozish: bitta tikda hammasi navbatda — keyingi harflar IME tahriridan oldin turadi.
    # Inglizcha so'zdan oldin: kontekst ("check" dan keyingi so'z) natijaga ta'sir qilmasin.
    { ClearBox }, { TypeKeys 'shoshma ok ' }, $w, $w, { Check 'typed ahead: shoshma ok' 'şoşma ok ' },
    { ClearBox }, { TypeKeys 'check ' }, $w, { Check 'english word kept' 'check ' },
    # 4) Tilmoch tartibi (avto o'chiq)
    { Settings @{ keymap = 1; layout = 'tilmoch'; auto_convert = 0 } }, $w, $w, $w, $w,
    { TypeKeys 'a ' }, $w,
    { ClearBox }, { TypeKeys "[;'" }, $w, { Check 'keymap: tilmoch letters' 'öşç' },
    { ClearBox }, { TypeKeys '{' }, $w, { Check 'keymap: shift level' 'Ö' },
    { ClearBox }, { AltGr '[' }, $w, { Check 'keymap: altgr level' '[' },
    { ClearBox }, { TypeKeys 'shahar ' }, $w, { Check 'keymap: auto off keeps sh' 'shahar ' },
    # 5) Kirill tartibi: avto yoqiq — yangi lotinga, o'chiq — kirill qoladi (x j q = ch o y)
    { Settings @{ keymap = 1; layout = 'kirill'; auto_convert = 1 } }, $w, $w, $w, $w,
    { TypeKeys 'a ' }, $w,
    { ClearBox }, { TypeKeys 'xjq ' }, $w, { Check 'kirill: auto on -> coy' 'çoy ' },
    { Settings @{ keymap = 1; layout = 'kirill'; auto_convert = 0 } }, $w, $w, $w, $w,
    { TypeKeys 'a ' }, $w,
    { ClearBox }, { TypeKeys 'xjq ' }, $w, { Check 'kirill: auto off stays cyrillic' 'чой ' },
    # 6) Tartib o'chiq — tugmalar odatdagidek
    { Settings @{ keymap = 0; auto_convert = 1 } }, $w, $w, $w, $w,
    { TypeKeys 'a ' }, $w,
    { ClearBox }, { TypeKeys '[x]' }, $w, { Check 'keymap off: plain keys' '[x]' }
)

$script:k = 0
$tick = {
    if ($script:k -lt $steps.Count) {
        try { & $steps[$script:k] } catch { $log.Add("ERROR step $($script:k): $_") }
        $script:k++
    } else {
        $script:timer.Stop()
        [IO.File]::WriteAllLines($Out, $log, (New-Object Text.UTF8Encoding $false))
        $win.Close()
    }
}
if ($Ui -eq 'wpf') {
    $win.Add_ContentRendered({
            $script:timer = New-Object System.Windows.Threading.DispatcherTimer
            $script:timer.Interval = [TimeSpan]::FromMilliseconds(350)
            $script:timer.Add_Tick($tick)
            $script:timer.Start()
        })
    $win.ShowDialog() | Out-Null
} else {
    $win.Add_Shown({
            $script:timer = New-Object System.Windows.Forms.Timer
            $script:timer.Interval = 350
            $script:timer.Add_Tick($tick)
            $script:timer.Start()
        })
    [System.Windows.Forms.Application]::Run($win)
}

Get-Content $Out -Encoding UTF8 | ForEach-Object { Write-Host $_ }
if (Select-String -Path $Out -Pattern '^(FAIL|ERROR)' -Quiet) { exit 1 }
