# Recreates the supplied square minimap picture as the mod icon (128x128).
param([string]$ModDir = (Split-Path $PSScriptRoot -Parent))
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$size = 128
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half

function Brush([int]$r, [int]$g2, [int]$b) {
    New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb($r, $g2, $b))
}
$white = Brush 255 255 255
$black = Brush 12 12 12
$gray = Brush 70 70 70
$grass = Brush 96 164 64
$grassDark = Brush 62 120 45
$grassLight = Brush 128 192 84
$tree = Brush 40 96 38
$red = Brush 224 32 32
$redDark = Brush 90 16 16

$g.FillRectangle($white, 0, 0, $size, $size)
$g.FillRectangle($black, 5, 5, 118, 118)
$g.FillRectangle($gray, 8, 8, 112, 112)

# Terrain with deterministic patches and tree clumps.
$g.FillRectangle($grass, 10, 10, 108, 108)
$rand = New-Object System.Random(42)
for ($i = 0; $i -lt 220; $i++) {
    $x = 11 + $rand.Next(0, 105)
    $y = 11 + $rand.Next(0, 105)
    $patch = $grassLight
    if ($i % 3 -eq 0) { $patch = $grassDark }
    $g.FillRectangle($patch, $x, $y, 2, 2)
}
for ($i = 0; $i -lt 80; $i++) {
    $x = 12 + $rand.Next(0, 100)
    $y = 12 + $rand.Next(0, 100)
    $g.FillRectangle($tree, $x, $y, 4, 3)
}

# Mob faces, from the same pictures used on the map.
$icons = @{
    'pig' = 'Pig.jfif'; 'sheep' = 'sheep.jfif'; 'cow' = 'Cow.jfif'
    'skeleton' = 'skeleton.jfif'; 'spider' = 'spider.jfif'
    'zombie' = 'Zombie.jfif'; 'chicken' = 'chicken.png'
}
$places = @(
    @('pig', 14, 12), @('sheep', 98, 16), @('cow', 12, 92),
    @('skeleton', 98, 90), @('spider', 58, 8), @('zombie', 56, 102),
    @('chicken', 10, 50), @('pig', 98, 52), @('skeleton', 30, 30),
    @('cow', 86, 38), @('pig', 38, 100), @('chicken', 110, 70),
    @('zombie', 14, 66), @('skeleton', 72, 104), @('sheep', 64, 16),
    @('pig', 106, 28), @('cow', 28, 74)
)
foreach ($p in $places) {
    $img = New-Object System.Drawing.Bitmap((Join-Path $ModDir "assets\icons\$($icons[$p[0]])"))
    $g.DrawImage($img, (New-Object System.Drawing.Rectangle($p[1], $p[2], 15, 15)), 0, 0, $img.Width, $img.Height, [System.Drawing.GraphicsUnit]::Pixel)
    $img.Dispose()
}

# Red player arrowhead in the middle.
$g.FillPolygon($redDark, @(
    (New-Object System.Drawing.Point(64, 50)), (New-Object System.Drawing.Point(76, 78)), (New-Object System.Drawing.Point(64, 72))
))
$g.FillPolygon($red, @(
    (New-Object System.Drawing.Point(64, 52)), (New-Object System.Drawing.Point(73, 76)), (New-Object System.Drawing.Point(64, 70))
))

# Blocky compass letters with a dark shadow, like the map's font.
$glyphs = @{
    'N' = @('10001','11001','10101','10011','10001','10001','10001')
    'E' = @('11111','10000','10000','11110','10000','10000','11111')
    'S' = @('01111','10000','10000','01110','00001','00001','11110')
    'W' = @('10001','10001','10001','10101','10101','11011','10001')
}
function Draw-Glyph([string]$ch, [int]$x, [int]$y) {
    $rows = $glyphs[$ch]
    for ($r = 0; $r -lt 7; $r++) {
        for ($c = 0; $c -lt 5; $c++) {
            if ($rows[$r][$c] -eq '1') {
                $g.FillRectangle($black, ($x + $c * 2 + 2), ($y + $r * 2 + 2), 2, 2)
                $g.FillRectangle($white, ($x + $c * 2), ($y + $r * 2), 2, 2)
            }
        }
    }
}
Draw-Glyph 'N' 57 -6
Draw-Glyph 'S' 57 118
Draw-Glyph 'W' -4 57
Draw-Glyph 'E' 118 57

$g.Dispose()
$bmp.Save((Join-Path $ModDir 'assets\textures\gui\icon.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
foreach ($b in @($white,$black,$gray,$grass,$grassDark,$grassLight,$tree,$red,$redDark)) { $b.Dispose() }
Write-Host 'Generated assets/textures/gui/icon.png'
