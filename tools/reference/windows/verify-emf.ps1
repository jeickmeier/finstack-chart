<# Independent Windows GDI playback qualification. Run in Windows PowerShell 5.1:
   .\verify-emf.ps1 -InputFile device-144.emf -OutputPng device-144-windows.png
   Optional ExpectedPng and MaxMeanAbsoluteError compare independently exported PNG.
   This never regenerates or replaces an expected image. #>
param(
    [Parameter(Mandatory=$true)][string]$InputFile,
    [Parameter(Mandatory=$true)][string]$OutputPng,
    [string]$ExpectedPng,
    [double]$MaxMeanAbsoluteError = 2.0
)
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'This verifier requires actual Windows GDI.' }
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class EmfPlayback {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int left,top,right,bottom; }
    [DllImport("gdi32.dll", CharSet=CharSet.Unicode, SetLastError=true)] public static extern IntPtr GetEnhMetaFile(string path);
    [DllImport("gdi32.dll", SetLastError=true)] public static extern bool PlayEnhMetaFile(IntPtr hdc, IntPtr emf, ref Rect bounds);
    [DllImport("gdi32.dll", SetLastError=true)] public static extern bool DeleteEnhMetaFile(IntPtr emf);
}
'@
$inputPath = (Resolve-Path -LiteralPath $InputFile).Path
$outputPath = [System.IO.Path]::GetFullPath($OutputPng)
if ($inputPath -eq $outputPath) { throw 'Input and output paths must differ.' }
if ($ExpectedPng -and (Resolve-Path -LiteralPath $ExpectedPng).Path -eq $outputPath) { throw 'Output must not overwrite the expected PNG.' }
$bytes = [System.IO.File]::ReadAllBytes($inputPath)
function U32([int]$offset) { [BitConverter]::ToUInt32($bytes,$offset) }
if ($bytes.Length -lt 108 -or (U32 0) -ne 1 -or (U32 40) -ne 0x464D4520 -or (U32 48) -ne $bytes.Length) { throw 'Invalid enhanced-metafile header.' }
$offset=0; $records=0; $last=0
while ($offset -lt $bytes.Length) {
    if ($offset + 8 -gt $bytes.Length) { throw 'Truncated record header.' }
    $last=U32 $offset; $size=U32 ($offset+4)
    if ($size -lt 8 -or $size % 4 -ne 0 -or $size -gt $bytes.Length-$offset) { throw 'Invalid record size.' }
    $offset += $size; $records++
}
if ($last -ne 14 -or $records -ne (U32 52)) { throw 'Record count or EOF mismatch.' }
$width=[int](U32 72); $height=[int](U32 76)
if ($width -lt 1 -or $height -lt 1 -or [long]$width*$height -gt 64000000) { throw 'Invalid or excessive device dimensions.' }
$emf=[EmfPlayback]::GetEnhMetaFile($inputPath)
if ($emf -eq [IntPtr]::Zero) { throw "GetEnhMetaFile failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())" }
$bitmap=New-Object System.Drawing.Bitmap($width,$height)
$graphics=[System.Drawing.Graphics]::FromImage($bitmap)
try {
    $graphics.Clear([System.Drawing.Color]::White)
    $hdc=$graphics.GetHdc()
    try {
        $rect=New-Object EmfPlayback+Rect
        $rect.right=$width; $rect.bottom=$height
        if (-not [EmfPlayback]::PlayEnhMetaFile($hdc,$emf,[ref]$rect)) { throw "PlayEnhMetaFile failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())" }
    } finally { $graphics.ReleaseHdc($hdc) }
    $bitmap.Save($outputPath,[System.Drawing.Imaging.ImageFormat]::Png)
    $mae=$null
    if ($ExpectedPng) {
        $expected=New-Object System.Drawing.Bitmap((Resolve-Path -LiteralPath $ExpectedPng).Path)
        try {
            if ($expected.Width -ne $width -or $expected.Height -ne $height) { throw 'Expected PNG dimensions disagree.' }
            [double]$sum=0
            for ($y=0; $y -lt $height; $y++) { for ($x=0; $x -lt $width; $x++) {
                $a=$bitmap.GetPixel($x,$y); $b=$expected.GetPixel($x,$y)
                $sum += [Math]::Abs([int]$a.R-$b.R)+[Math]::Abs([int]$a.G-$b.G)+[Math]::Abs([int]$a.B-$b.B)
            } }
            $mae=$sum/(3.0*$width*$height)
            if ($mae -gt $MaxMeanAbsoluteError) { throw "Independent PNG mismatch: MAE=$mae exceeds $MaxMeanAbsoluteError" }
        } finally { $expected.Dispose() }
    }
    [ordered]@{input=$inputPath;output=$outputPath;bytes=$bytes.Length;records=$records;width=$width;height=$height;gdi_playback='pass';mean_absolute_rgb_error=$mae;os=[Environment]::OSVersion.ToString()} | ConvertTo-Json
} finally { $graphics.Dispose(); $bitmap.Dispose(); [void][EmfPlayback]::DeleteEnhMetaFile($emf) }
