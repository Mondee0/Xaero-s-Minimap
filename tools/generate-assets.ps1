# Regenerates the minimap mod's GUI textures (256 palette colors + backgrounds + icon).
# powershell -File tools/generate-assets.ps1
param(
    [string]$Out = (Join-Path $PSScriptRoot '..\assets\textures\gui')
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$guiDir = [System.IO.Path]::GetFullPath($Out)
$palDir = Join-Path $guiDir 'minimap'
New-Item -ItemType Directory -Force -Path $palDir | Out-Null

function New-Bitmap([int]$w, [int]$h) {
    $b = New-Object System.Drawing.Bitmap($w, $h)
    return @{ Bmp = $b; G = [System.Drawing.Graphics]::FromImage($b) }
}
function Save-Png($bmp, [string]$path) { $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png) }
function Fill-Rect($g, $brush, [int]$x, [int]$y, [int]$w, [int]$h) { $g.FillRectangle($brush, $x, $y, $w, $h) }
function Write-Palette([int]$index, [int]$r, [int]$g, [int]$b, [int]$a) {
    $pair = New-Bitmap 1 1
    try {
        $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb($a, $r, $g, $b))
        try { Fill-Rect $pair.G $brush 0 0 1 1 } finally { $brush.Dispose() }
        Save-Png $pair.Bmp (Join-Path $palDir ('p{0:D3}.png' -f $index))
    } finally { $pair.G.Dispose(); $pair.Bmp.Dispose() }
}

# 32 terrain bases x 4 shades = indices 0..127.
$bases = @(
    @( 96, 164,  64), #  0 grass
    @(128, 192,  84), #  1 grass light
    @( 62, 120,  45), #  2 grass dark
    @( 44, 104,  42), #  3 leaves
    @( 30,  76,  32), #  4 leaves dark
    @( 68, 140,  58), #  5 leaves light
    @(132,  84,  50), #  6 dirt
    @( 96,  60,  36), #  7 dirt dark
    @(130, 130, 130), #  8 stone
    @( 96,  96,  96), #  9 stone dark
    @(164, 164, 164), # 10 stone light
    @(112, 112, 112), # 11 cobble
    @(226, 214, 160), # 12 sand
    @(242, 232, 190), # 13 sand light
    @(204, 188, 138), # 14 sandstone
    @(138, 130, 124), # 15 gravel
    @( 48, 102, 200), # 16 water
    @( 32,  70, 150), # 17 water dark
    @( 72, 132, 222), # 18 water light
    @(246, 250, 252), # 19 snow
    @(168, 212, 232), # 20 ice
    @( 96,  64,  40), # 21 wood
    @(158, 110,  66), # 22 planks
    @( 68,  44,  26), # 23 bark
    @(206,  70, 116), # 24 flowers
    @(232, 212,  74), # 25 yellow flowers
    @(184,  48,  48), # 26 red
    @(224,  96,  32), # 27 lava
    @(100,  42,  42), # 28 nether
    @(207, 201, 160), # 29 end
    @(196, 172,  92), # 30 ore
    @( 68, 110,  50)  # 31 moss
)
$shades = @(0.62, 0.85, 1.0, 1.22)

for ($base = 0; $base -lt $bases.Count; $base++) {
    $c = $bases[$base]
    for ($shade = 0; $shade -lt 4; $shade++) {
        $f = $shades[$shade]
        $r = [Math]::Min(255, [int][Math]::Round($c[0] * $f))
        $g = [Math]::Min(255, [int][Math]::Round($c[1] * $f))
        $b = [Math]::Min(255, [int][Math]::Round($c[2] * $f))
        Write-Palette ($base * 4 + $shade) $r $g $b 255
    }
}

# Specials and sprite colors: 128..160.
$specials = @(
    @(  0,   0,   0,   0), # 128 transparent
    @( 43,  43,  43, 255), # 129 undiscovered
    @( 16,  16,  16, 255), # 130 black
    @(255, 255, 255, 255), # 131 white
    @( 12,  12,  12, 255), # 132 frame outer
    @(200, 200, 200, 255), # 133 frame light
    @( 74,  74,  74, 255), # 134 frame dark
    @(122, 122, 122, 255), # 135 frame mid
    @(224,  48,  48, 255), # 136 hostile
    @( 80, 208,  80, 255), # 137 passive
    @(240, 208,  32, 255), # 138 item
    @( 48, 208, 224, 255), # 139 player
    @( 48,  96, 240, 255), # 140 wp blue
    @( 48, 208, 112, 255), # 141 wp green
    @(240, 144,  32, 255), # 142 wp orange
    @(224,  80, 160, 255), # 143 wp pink
    @(240, 208,  40, 255), # 144 waypoint yellow
    @(224,  32,  32, 255), # 145 arrow red
    @( 90,  16,  16, 255), # 146 arrow dark
    @( 28,  28,  28, 255), # 147 sprite dark
    @(245, 245, 240, 255), # 148 sprite white
    @( 95, 158,  60, 255), # 149 sprite green
    @( 63, 122,  46, 255), # 150 sprite green dark
    @(230, 162, 176, 255), # 151 sprite pink
    @(168, 107, 120, 255), # 152 sprite pink dark
    @(107,  74,  46, 255), # 153 sprite brown
    @( 74,  48,  32, 255), # 154 sprite brown dark
    @(192,  42,  42, 255), # 155 sprite red
    @(224, 144,  32, 255), # 156 sprite orange
    @(240, 216,  74, 255), # 157 sprite yellow
    @(154, 154, 154, 255), # 158 sprite gray
    @( 90,  90,  90, 255), # 159 sprite gray dark
    @(200, 200, 200, 255), # 160 sprite light gray
    @(160, 122,  90, 255), # 161 sprite tan
    @(120,  88,  64, 255), # 162 sprite tan dark
    @(190, 158, 126, 255), # 163 sprite tan light
    @(224, 160, 160, 255), # 164 sprite snout
    @(176, 112, 112, 255), # 165 sprite snout dark
    @( 74,  58,  48, 255), # 166 sprite brick
    @( 48,  36,  30, 255), # 167 sprite brick dark
    @(230,  16,  16, 255), # 168 sprite eye red
    @(138,   0,   0, 255), # 169 sprite eye red dark
    @(122, 170,  80, 255)  # 170 sprite green light
)
for ($i = 0; $i -lt $specials.Count; $i++) {
    $c = $specials[$i]
    Write-Palette (128 + $i) $c[0] $c[1] $c[2] $c[3]
}
for ($i = 160; $i -lt 256; $i++) { Write-Palette $i 0 0 0 0 }

# Backing for the fullscreen map.
$pair = New-Bitmap 16 16
try {
    $fill = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(235, 12, 14, 20))
    $edge = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 8, 8, 10))
    $edge2 = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 70, 70, 78))
    try {
        Fill-Rect $pair.G $fill 0 0 16 16
        Fill-Rect $pair.G $edge 0 0 16 2
        Fill-Rect $pair.G $edge 0 14 16 2
        Fill-Rect $pair.G $edge 0 0 2 16
        Fill-Rect $pair.G $edge 14 0 2 16
        Fill-Rect $pair.G $edge2 2 2 12 1
    } finally { $fill.Dispose(); $edge.Dispose(); $edge2.Dispose() }
    Save-Png $pair.Bmp (Join-Path $guiDir 'map_bg.png')
} finally { $pair.G.Dispose(); $pair.Bmp.Dispose() }
Set-Content -Path (Join-Path $guiDir 'map_bg.gui') -Value "scaling = nine_slice`nborder = 2`nsize = 16 16" -NoNewline

# The mod icon is a recreation of the supplied minimap picture; see generate-logo.ps1.

Write-Host "Generated 256 palette textures and backgrounds in $guiDir"
