# Generate Tauri icons from public/logos/logo-light.png
# Produces PNGs in src-tauri/icons and an icon.ico if ImageMagick is available.

param(
    [string]$WorkspaceRoot = "C:\Users\enric\Desktop\G4",
    [string]$Source = "ruggine_client/public/logos/logo-light.png",
    [string]$DestDir = "ruggine_client/src-tauri/icons"
)

$srcPath = Join-Path $WorkspaceRoot $Source
$dest = Join-Path $WorkspaceRoot $DestDir

if (-not (Test-Path $srcPath)) {
    Write-Error "Source file not found: $srcPath"
    exit 2
}

if (-not (Test-Path $dest)) {
    New-Item -ItemType Directory -Path $dest | Out-Null
}

# Sizes to create (png)
$sizes = @(32, 44, 71, 89, 107, 128, 142, 150, 284, 310, 512)

# Helper to run magick if available
function Use-ImageMagick {
    $magick = Get-Command magick -ErrorAction SilentlyContinue
    if ($magick) { return $true }
    return $false
}

if (Use-ImageMagick) {
    Write-Host "Using ImageMagick to generate icons..."
    foreach ($s in $sizes) {
        $out = Join-Path $dest ("{0}x{0}.png" -f $s)
        magick convert "$srcPath" -resize ${s}x${s} "$out"
        Write-Host "Created $out"
    }

    # Create a multi-size icon.ico from a selection of sizes
    $icoSizes = @(16,32,48,64,128,256)
    $tmpFiles = @()
    foreach ($s in $icoSizes) {
        $tmp = Join-Path $env:TEMP ("ruggine_icon_${s}.png")
        magick convert "$srcPath" -resize ${s}x${s} "$tmp"
        $tmpFiles += $tmp
    }
    $icoOut = Join-Path $dest "icon.ico"
    magick convert $tmpFiles -colors 256 "$icoOut"
    Write-Host "Created $icoOut"

    # Clean temp files
    foreach ($f in $tmpFiles) { Remove-Item $f -ErrorAction SilentlyContinue }

    Write-Host "Icon generation complete."
} else {
    Write-Warning "ImageMagick (magick) not found in PATH.\nScript will create PNGs using System.Drawing if available. ICO generation will be skipped."
    try {
        Add-Type -AssemblyName System.Drawing
    } catch {
        Write-Error "System.Drawing not available. Install ImageMagick or run on Windows with .NET."
        exit 3
    }

    foreach ($s in $sizes) {
        $out = Join-Path $dest ("{0}x{0}.png" -f $s)
        $img = [System.Drawing.Image]::FromFile($srcPath)
        $bmp = New-Object System.Drawing.Bitmap $img, $s, $s
        $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
        $img.Dispose()
        Write-Host "Created $out"
    }

    Write-Host "PNG icon generation complete (ICO skipped). Install ImageMagick to produce .ico and .icns files."
}
