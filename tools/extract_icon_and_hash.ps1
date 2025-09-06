param(
    [string]$ExePath = "C:\Users\enric\Desktop\G4\ruggine_client\target\debug\ruggine_client.exe",
    [string]$SrcIcon = "C:\Users\enric\Desktop\G4\ruggine_client\src-tauri\icons\icon.ico",
    [string]$Out = "C:\Users\enric\Desktop\G4\temp_extracted_icon.ico"
)

if (-not (Test-Path $ExePath)) {
    Write-Error "Exe not found: $ExePath"
    exit 2
}

Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon($ExePath)
if ($null -eq $icon) {
    Write-Error "No icon extracted from $ExePath"
    exit 3
}

$fs = New-Object System.IO.FileStream($Out,[System.IO.FileMode]::Create)
$icon.Save($fs)
$fs.Close()
Write-Output "SavedIcon:$Out"

Write-Output "Hash of extracted icon:"
Get-FileHash $Out -Algorithm SHA256 | Format-List

if (Test-Path $SrcIcon) {
    Write-Output "Hash of source icon ($SrcIcon):"
    Get-FileHash $SrcIcon -Algorithm SHA256 | Format-List
} else {
    Write-Output "Source icon not found at $SrcIcon"
}
