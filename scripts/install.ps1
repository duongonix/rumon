param(
  [string]$Version = "latest",
  [string]$Repo = $env:RUMON_REPO,
  [string]$InstallDir = $env:RUMON_INSTALL_DIR
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Repo)) {
  $Repo = "duongonix/rumon"
}

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
  $InstallDir = Join-Path $HOME ".local\bin"
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
switch ($arch) {
  "x64" { $archName = "x86_64" }
  "arm64" { $archName = "aarch64" }
  default { throw "Unsupported architecture: $arch" }
}

if ($Version -eq "latest") {
  $latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
  $Version = $latest.tag_name
}

if ([string]::IsNullOrWhiteSpace($Version)) {
  throw "Could not resolve release version. Pass it explicitly, e.g. .\install.ps1 -Version v0.1.0"
}

$asset = "rumon-$Version-windows-$archName.zip"
$url = "https://github.com/$Repo/releases/download/$Version/$asset"

$workDir = Join-Path $env:TEMP ("rumon-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $workDir -Force | Out-Null
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

try {
  $zipPath = Join-Path $workDir $asset
  Write-Host "Installing Rumon from $url"
  Invoke-WebRequest -Uri $url -OutFile $zipPath

  Expand-Archive -Path $zipPath -DestinationPath $workDir -Force

  $expectedDir = Join-Path $workDir ("rumon-$Version-windows-$archName")
  if (Test-Path (Join-Path $expectedDir "rumon.exe")) {
    $pkgDir = $expectedDir
  }
  elseif (Test-Path (Join-Path $workDir "rumon.exe")) {
    $pkgDir = $workDir
  }
  else {
    $candidate = Get-ChildItem -Path $workDir -Directory |
      Where-Object { Test-Path (Join-Path $_.FullName "rumon.exe") } |
      Select-Object -First 1

    if ($null -eq $candidate) {
      throw "Package layout not recognized: rumon.exe was not found after extraction"
    }

    $pkgDir = $candidate.FullName
  }

  Copy-Item -Force (Join-Path $pkgDir "rumon.exe") (Join-Path $InstallDir "rumon.exe")

  Write-Host "Installed:"
  Write-Host "  $(Join-Path $InstallDir 'rumon.exe')"

  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  $pathParts = @()
  if (-not [string]::IsNullOrWhiteSpace($userPath)) {
    $pathParts = $userPath.Split(';') | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
  }
  if ($pathParts -notcontains $InstallDir) {
    $newPath = (($pathParts + $InstallDir) -join ';').Trim(';')
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host ""
    Write-Host "Added to User PATH: $InstallDir"
    Write-Host "Open a new terminal to use 'rumon' from anywhere."
  }

  Write-Host ""
  Write-Host "Verify:"
  Write-Host "  rumon --version"
}
finally {
  if (Test-Path $workDir) {
    Remove-Item -LiteralPath $workDir -Recurse -Force
  }
}
