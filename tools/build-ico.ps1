param(
  [string]$Source = "C:\Users\enric\Desktop\G4\ruggine_client\public\logos\logo-light.png",
  [string]$OutIco = "C:\Users\enric\Desktop\G4\ruggine_client\src-tauri\icons\icon.ico"
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Source)) { Write-Error "Source not found: $Source"; exit 2 }
Add-Type -AssemblyName System.Drawing

# Desired sizes
$sizes = @(16,24,32,48,64,128,256)
$tmpPngs = @()

$img = [System.Drawing.Image]::FromFile($Source)
foreach ($s in $sizes) {
  $bmp = New-Object System.Drawing.Bitmap $s, $s
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.Clear([System.Drawing.Color]::Transparent)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.DrawImage($img, 0,0,$s,$s)
  $g.Dispose()
  $tmp = Join-Path $env:TEMP ("ruggine_${s}.png")
  $bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  $tmpPngs += $tmp
}
$img.Dispose()

# Build ICO manually (PNG entries allowed)
$fs = [System.IO.File]::Open($OutIco, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
$bw = New-Object System.IO.BinaryWriter($fs)

# ICONDIR header
$bw.Write([UInt16]0)      # reserved
$bw.Write([UInt16]1)      # type (1=icon)
$bw.Write([UInt16]$sizes.Count) # count

# First pass: collect PNG bytes and compute offsets
$pngBytesList = @()
$offset = 6 + (16 * $sizes.Count) # header + directory entries
foreach ($idx in 0..($sizes.Count-1)) {
  $bytes = [System.IO.File]::ReadAllBytes($tmpPngs[$idx])
  $pngBytesList += ,@($sizes[$idx], $bytes, $offset)
  $offset += $bytes.Length
}

# Write directory entries
foreach ($entry in $pngBytesList) {
  $origSize = [int]$entry[0]
  $w = if ($origSize -ge 256) { 0 } else { [byte]$origSize }
  $h = $w
  $bytes = $entry[1]
  $ofs = [Int32]$entry[2]
  $bw.Write([byte]$w)      # width (0 means 256)
  $bw.Write([byte]$h)      # height
  $bw.Write([byte]0)       # color count
  $bw.Write([byte]0)       # reserved
  $bw.Write([UInt16]0)     # planes
  $bw.Write([UInt16]32)    # bit count
  $bw.Write([Int32]$bytes.Length) # bytes in resource
  $bw.Write([Int32]$ofs)   # offset
}

# Write image data
foreach ($entry in $pngBytesList) {
  $bw.Write($entry[1])
}

$bw.Flush(); $bw.Close(); $fs.Close()

Write-Host "Created ICO: $OutIco"
Get-FileHash $OutIco -Algorithm SHA256 | Format-List

# Cleanup temps
foreach ($t in $tmpPngs) { Remove-Item $t -ErrorAction SilentlyContinue }
