param([string]$Src = "$PSScriptRoot\..\public\icon-512.png", [string]$Out = "$PSScriptRoot\..\public\apple-touch-icon.png")
# iOS fills transparent pixels with black on the home screen, so the touch icon needs an opaque background.
Add-Type -AssemblyName System.Drawing
$size = 180
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(255, 251, 246, 238))   # site background #fbf6ee
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
$icon = [System.Drawing.Image]::FromFile((Resolve-Path $Src))
$inner = 150
$pad = ($size - $inner) / 2
$g.DrawImage($icon, $pad, $pad, $inner, $inner)
$icon.Dispose()
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
"saved $Out"
