param(
  [Parameter(Mandatory = $true)]
  [string]$Tag,
  [string]$Remote = "origin",
  [switch]$SkipChecks,
  [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"

function Fail($msg) {
  Write-Error $msg
  exit 1
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
  Fail "git is required"
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Fail "cargo is required"
}

$version = $Tag
if ($version.StartsWith("v")) {
  $version = $version.Substring(1)
}

if ($version -notmatch '^\d+\.\d+\.\d+([\-+][0-9A-Za-z\.\-]+)?$') {
  Fail "invalid tag/version: '$Tag'. Expected like v0.1.0"
}

$repoRoot = (git rev-parse --show-toplevel).Trim()
Set-Location $repoRoot

$status = git status --porcelain
if (-not $AllowDirty -and -not [string]::IsNullOrWhiteSpace($status)) {
  Fail "working tree is not clean. Commit/stash first, or pass -AllowDirty"
}

Write-Host "==> Updating workspace version to $version"
$cargoPath = Join-Path $repoRoot "Cargo.toml"
$content = Get-Content -LiteralPath $cargoPath -Raw
$pattern = '(?ms)(^\s*\[workspace\.package\]\s*.*?^\s*version\s*=\s*")([^"]+)(")'
$currentMatch = [regex]::Match($content, $pattern)
if (-not $currentMatch.Success) {
  Fail "failed to find [workspace.package] version in Cargo.toml"
}
$currentVersion = $currentMatch.Groups[2].Value
$updated = [regex]::Replace($content, $pattern, "`${1}$version`${3}", 1)
if ($updated -eq $content -and $currentVersion -eq $version) {
  Write-Host "  workspace version is already $version"
}
elseif ($updated -eq $content) {
  Fail "failed to update workspace package version in Cargo.toml"
}
Set-Content -LiteralPath $cargoPath -Value $updated -NoNewline

Write-Host "==> Refreshing Cargo.lock"
cargo metadata --format-version 1 | Out-Null

if (-not $SkipChecks) {
  Write-Host "==> Running checks"
  cargo fmt --all --check
  cargo check --workspace --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
}

Write-Host "==> Committing release changes"
git add Cargo.toml Cargo.lock
$staged = git diff --cached --name-only
if ([string]::IsNullOrWhiteSpace($staged)) {
  Write-Host "  no version changes to commit"
}
else {
  git commit -m "release: $Tag"
}

Write-Host "==> Tagging and pushing"
git tag $Tag
git push $Remote HEAD
git push $Remote $Tag

Write-Host ""
Write-Host "Release tag pushed: $Tag"
Write-Host "GitHub release workflow should start automatically."
