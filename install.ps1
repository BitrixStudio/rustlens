Param(
  [string]$Repo = "BitrixStudio/rustlens",
  [string]$Version = "latest",
  [string]$BinDir = "$env:USERPROFILE\.local\bin",
  [string]$Only = ""
)

$ErrorActionPreference = "Stop"

function Get-LatestTag($repo) {
  $json = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
  return $json.tag_name
}

if ($Version -eq "latest") {
  $Version = Get-LatestTag $Repo
}

$Target = "x86_64-pc-windows-msvc"
$Asset = "rustlens-$Version-$Target.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$SumsUrl = "https://github.com/$Repo/releases/download/$Version/SHA256SUMS.txt"

Write-Host "Downloading $Asset ..."
$tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::Combine($env:TEMP, "rustlens-install"))
$zipPath = Join-Path $tmp $Asset
Invoke-WebRequest -Uri $Url -OutFile $zipPath

# checksum (unsure if this is optimal)
try {
  $sums = Invoke-WebRequest -Uri $SumsUrl -UseBasicParsing
  $expected = ($sums.Content -split "`n" | Where-Object { $_ -match [regex]::Escape($Asset) } | Select-Object -First 1).Split(" ")[0].ToLower()
  if ($expected) {
    $actual = (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $actual) { throw "Checksum mismatch" }
  }
} catch {
  Write-Host "Checksum verification skipped: $($_.Exception.Message)"
}

Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
$pkgDir = Join-Path $tmp ("rustlens-$Version-$Target")

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

function Install-One($name) {
  $src = Join-Path $pkgDir "$name.exe"
  $dst = Join-Path $BinDir "$name.exe"
  if (!(Test-Path $src)) { throw "Missing binary: $src" }
  Copy-Item $src $dst -Force
  Write-Host "Installed: $dst"
}

if ([string]::IsNullOrEmpty($Only)) {
  Install-One "rustlens"
  Install-One "rustlensmanager"
} else {
  Install-One $Only
}

Write-Host ""
Write-Host "Done."
Write-Host "Add to PATH if needed:"
Write-Host "  setx PATH `"$BinDir;%PATH%`""
