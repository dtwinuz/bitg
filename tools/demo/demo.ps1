# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

<#
.SYNOPSIS
  README'dagi demo GIF uchun kadrlar: o'rnatilgan bitg IME'da haqiqiy yozish yozib olinadi.

  Oyna ochiladi, bitg faollashtiriladi, gaplar SendInput bilan odam tezligida teriladi va har ~40 ms da oyna
  skrinshoti olinadi. Oxirida matn maydonidagi natija `bitg.exe` (o'sha yadro) natijasi bilan solishtiriladi:
  mos kelmasa skript xato bilan tugaydi va GIF yasalmaydi. Kadrlar: <Out>\frames\NNNNN.png, vaqtlari
  <Out>\frames.tsv. GIF'ni tools/demo/make_gif.py yig'adi.

  FAQAT CI VM'da (GitHub Actions, windows-latest): SendInput tugmalari fokusdagi har qanday oynaga ketadi.

.EXAMPLE
  powershell -File tools/demo/demo.ps1 -Out $env:RUNNER_TEMP\demo
#>
param(
    [string]$Out = (Join-Path ([IO.Path]::GetTempPath()) 'bitg-demo'),
    [string]$Cli = 'C:\Program Files\bitg\bitg.exe'
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw "Faqat CI VM uchun (SendInput). GITHUB_ACTIONS=true emas." }

# Teriladigan gaplar: eski lotin (apostrof bilan), inglizcha aralash gap, g' va ch.
# Fayl ASCII: Windows PowerShell 5.1 BOM'siz UTF-8 ni ANSI deb o'qiydi.
$lines = @(
    "O'zbekiston shaharlari: Toshkent, Samarqand ",
    "Please check the schedule, keyin choy ichamiz ",
    "Tog'dagi bog'lar juda chiroyli "
)

Add-Type -AssemblyName PresentationFramework, System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System; using System.Runtime.InteropServices; using System.Drawing; using System.Drawing.Imaging;
[ComImport, Guid("71c6e74c-0f28-11d8-a82a-00065b84435c"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface ITfInputProcessorProfileMgr {
    [PreserveSig] int ActivateProfile(uint type, ushort langid, ref Guid clsid, ref Guid profile, IntPtr hkl, uint flags);
}
public static class Demo {
    [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] public struct U { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
    [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public U u; }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);

    public static int ActivateBitg() {
        var t = Type.GetTypeFromCLSID(new Guid("33C53A50-F456-4884-B049-85FD643ECFED"));
        var pm = (ITfInputProcessorProfileMgr)Activator.CreateInstance(t);
        var clsid = new Guid("e37d4f70-f82a-435e-af8b-82e264391062");
        var prof = new Guid("bed48fc8-8690-4712-bbe0-1fd3284ce7b7");
        return pm.ActivateProfile(1, 0x0443, ref clsid, ref prof, IntPtr.Zero, 0x10000000);
    }
    public static uint Scan(ushort scan, bool up) {
        var i = new INPUT { type = 1 };
        i.u.ki.wScan = scan;
        i.u.ki.dwFlags = 0x8u | (up ? 0x2u : 0u);
        return SendInput(1, new[] { i }, Marshal.SizeOf(typeof(INPUT)));
    }
    // Oynaning ekrandagi tasviri (DWM kompozitsiyasidan keyin, IME almashtirgan matn bilan).
    public static void Shot(IntPtr h, string path) {
        RECT r; GetWindowRect(h, out r);
        using (var bmp = new Bitmap(r.R - r.L, r.B - r.T, PixelFormat.Format24bppRgb))
        using (var g = Graphics.FromImage(bmp)) {
            g.CopyFromScreen(r.L, r.T, 0, 0, bmp.Size, CopyPixelOperation.SourceCopy);
            bmp.Save(path, ImageFormat.Png);
        }
    }
}
'@

# --- US tartibi: belgi -> skan-kod (katta harf va : " = Shift) ---
$scan = @{}
$i = 0; foreach ($c in 'qwertyuiop[]'.ToCharArray()) { $scan[[string]$c] = 0x10 + $i; $i++ }
$i = 0; foreach ($c in "asdfghjkl;'".ToCharArray()) { $scan[[string]$c] = 0x1E + $i; $i++ }
$i = 0; foreach ($c in 'zxcvbnm,./'.ToCharArray()) { $scan[[string]$c] = 0x2C + $i; $i++ }
$scan[' '] = 0x39
$shifted = @{ ':' = ';'; '"' = "'" }
$SHIFT = 0x2A; $ENTER = 0x1C

function Tap([int]$s) { [void][Demo]::Scan($s, $false); [void][Demo]::Scan($s, $true) }
function Key([string]$c) {
    if ($c -eq "`n") { Tap $ENTER; return }
    $up = ($c -cmatch '[A-Z]') -or $shifted.ContainsKey($c)
    $base = if ($shifted.ContainsKey($c)) { $shifted[$c] } else { $c.ToLower() }
    $s = $scan[$base]
    if ($null -eq $s) { throw "skan-kod yo'q: $c" }
    if ($up) { [void][Demo]::Scan($SHIFT, $false) }
    Tap $s
    if ($up) { [void][Demo]::Scan($SHIFT, $true) }
}

# --- Sozlamalar: standart (avto o'girish yoqiq, tartib o'chiq) ---
$cfgDir = Join-Path $env:APPDATA 'bitg'
New-Item -ItemType Directory -Force $cfgDir | Out-Null
[IO.File]::WriteAllText((Join-Path $cfgDir 'settings.ini'),
    "enabled=1`nauto_convert=1`nkeymap=0`nundo_backspace=1`nundo_remember=0`nfallback=1`ndisabled_apps=`n")
Remove-Item (Join-Path $cfgDir 'exceptions.txt') -ErrorAction SilentlyContinue

# --- Kutilgan natija: o'sha yadro, CLI orqali (chiqishi UTF-8) ---
[Console]::OutputEncoding = New-Object Text.UTF8Encoding $false
$want = ($lines | ForEach-Object {
        $o = & $Cli $_.TrimEnd()
        if ($LASTEXITCODE) { throw "bitg.exe: $LASTEXITCODE" }
        "$o "
    }) -join "`r`n"

# --- Jadval: har tugma qachon bosiladi (ms). Tasodif urug'i qotirilgan: har yozuv bir xil. ---
$rng = New-Object Random 2026
$plan = New-Object System.Collections.Generic.List[object]
$t = 900
for ($n = 0; $n -lt $lines.Count; $n++) {
    if ($n -gt 0) { $plan.Add(@($t, "`n")); $t += 380 }
    foreach ($ch in $lines[$n].ToCharArray()) {
        $plan.Add(@($t, [string]$ch))
        $t += 58 + $rng.Next(0, 46)
        if ($ch -eq ' ') { $t += 110 }
        if ($ch -eq ',' -or $ch -eq ':') { $t += 160 }
    }
    $t += 1100
}
$end = $t + 1700

# --- Oyna: bitg.uz uslubi (tinch sirt, 1px chiziq, bitta aksent #2f62c0) ---
[xml]$xaml = @'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        Title="bitg demo" Width="960" Height="226" WindowStyle="None" ResizeMode="NoResize"
        WindowStartupLocation="CenterScreen" Background="#FFFFFF" Topmost="True" UseLayoutRounding="True"
        TextOptions.TextFormattingMode="Display" TextOptions.TextRenderingMode="Grayscale">
  <Border BorderBrush="#E3E6EB" BorderThickness="1">
    <DockPanel>
      <Border DockPanel.Dock="Top" Height="44" Background="#F6F7F9" BorderBrush="#E3E6EB" BorderThickness="0,0,0,1">
        <Grid Margin="18,0">
          <StackPanel Orientation="Horizontal" VerticalAlignment="Center">
            <Ellipse Width="11" Height="11" Fill="#D9DCE1" Margin="0,0,7,0"/>
            <Ellipse Width="11" Height="11" Fill="#D9DCE1" Margin="0,0,7,0"/>
            <Ellipse Width="11" Height="11" Fill="#D9DCE1"/>
          </StackPanel>
          <TextBlock Text="Istalgan dasturda: eskicha yozing, so'z tugashi bilan yangi alifbo" HorizontalAlignment="Center"
                     VerticalAlignment="Center" FontFamily="Segoe UI" FontSize="13" Foreground="#6B7280"/>
          <Border HorizontalAlignment="Right" VerticalAlignment="Center" CornerRadius="10" Padding="10,2"
                  Background="#EAF0FB" BorderBrush="#C9D7F3" BorderThickness="1">
            <TextBlock Text="bitg  &#246;" FontFamily="Segoe UI Semibold" FontSize="12" Foreground="#2F62C0"/>
          </Border>
        </Grid>
      </Border>
      <TextBox Name="Box" FontFamily="Segoe UI" FontSize="27" Foreground="#16181D" Background="#FFFFFF"
               BorderThickness="0" Padding="30,22,30,10" AcceptsReturn="True" CaretBrush="#2F62C0"
               SelectionBrush="#8FB0EA" TextWrapping="NoWrap" VerticalScrollBarVisibility="Disabled"/>
    </DockPanel>
  </Border>
</Window>
'@
$win = [Windows.Markup.XamlReader]::Load((New-Object System.Xml.XmlNodeReader $xaml))
$tb = $win.FindName('Box')

$frames = Join-Path $Out 'frames'
New-Item -ItemType Directory -Force $frames | Out-Null
Get-ChildItem $frames -Filter *.png -ErrorAction SilentlyContinue | Remove-Item
$times = New-Object System.Collections.Generic.List[string]
$log = New-Object System.Collections.Generic.List[string]
$script:h = [IntPtr]::Zero
$script:phase = 0
$script:next = 0
$script:frame = 0
$sw = New-Object Diagnostics.Stopwatch
$warm = New-Object Diagnostics.Stopwatch

function Focus {
    $win.Activate() | Out-Null
    [void][Demo]::SetForegroundWindow($script:h)
    $tb.Focus() | Out-Null
}

$tick = {
    try {
        switch ($script:phase) {
            0 {
                # bitg'ni faollashtirish; IME sozlamani birinchi tugmada o'qiydi, shuning uchun "a " bilan qizdiramiz
                $script:h = (New-Object System.Windows.Interop.WindowInteropHelper $win).Handle
                Focus
                $log.Add("ActivateProfile hr=0x{0:x}" -f [Demo]::ActivateBitg())
                Focus
                $warm.Start(); $script:phase = 1
            }
            1 {
                if ($warm.ElapsedMilliseconds -gt 1500) { Key 'a'; Key ' '; $warm.Restart(); $script:phase = 2 }
            }
            2 {
                if ($warm.ElapsedMilliseconds -gt 900) {
                    $tb.Clear()
                    $log.Add("foreground=" + ([Demo]::GetForegroundWindow() -eq $script:h))
                    $sw.Start(); $script:phase = 3
                }
            }
            3 {
                $now = $sw.ElapsedMilliseconds
                while ($script:next -lt $plan.Count -and $plan[$script:next][0] -le $now) {
                    Key $plan[$script:next][1]; $script:next++
                }
                $p = Join-Path $frames ('{0:D5}.png' -f $script:frame)
                [Demo]::Shot($script:h, $p)
                # qavs shart: metod chaqiruvida vergul -f ning emas, Add() ning argumentini ajratadi
                $times.Add(("{0}`t{1}" -f $script:frame, $sw.ElapsedMilliseconds))
                $script:frame++
                if ($now -ge $end) { $script:phase = 4 }
            }
            4 {
                $script:timer.Stop()
                $got = $tb.Text
                if ($got -ceq $want) { $log.Add("PASS demo text") }
                else { $log.Add("FAIL demo text`n got=[$got]`nwant=[$want]") }
                $log.Add("frames=$($script:frame) ms=$($sw.ElapsedMilliseconds)")
                $win.Close()
            }
        }
    } catch {
        $log.Add("ERROR phase $($script:phase): $_")
        $script:timer.Stop(); $win.Close()
    }
}

$win.Add_ContentRendered({
        $script:timer = New-Object System.Windows.Threading.DispatcherTimer
        $script:timer.Interval = [TimeSpan]::FromMilliseconds(40)
        $script:timer.Add_Tick($tick)
        $script:timer.Start()
    })
$win.ShowDialog() | Out-Null

$utf8 = New-Object Text.UTF8Encoding $false
[IO.File]::WriteAllLines((Join-Path $Out 'frames.tsv'), $times, $utf8)
[IO.File]::WriteAllLines((Join-Path $Out 'demo.txt'), $log, $utf8)
$log | ForEach-Object { Write-Host $_ }
if ($log | Where-Object { $_ -match '^(FAIL|ERROR)' }) { exit 1 }
