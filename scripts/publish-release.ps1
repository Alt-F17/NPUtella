$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$version = (Get-Content -Path (Join-Path $root "Cargo.toml") | Select-String -Pattern '^version\s*=\s*"(.*?)"' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Select-Object -First 1)
if (-not $version) { throw "Could not read version from Cargo.toml" }
if ($version -notmatch '^\d+\.\d+\.\d+$') {
    throw "MSI Package Version must be numeric major.minor.patch; Cargo.toml has '$version'"
}

$gh = Get-Command gh.exe -ErrorAction SilentlyContinue
if (-not $gh) { throw "gh.exe not found. Install GitHub CLI or publish from the tag workflow." }
& $gh.Source auth status | Out-Null
if ($LASTEXITCODE -ne 0) { throw "GitHub CLI is not authenticated. Run: gh auth login" }

$tag = "v$version"
$installer = Join-Path $root "dist\NPUtella-$version-SnapdragonXPlus-NPU.msi"
if (-not (Test-Path $installer)) {
    & (Join-Path $PSScriptRoot "build-installer.ps1")
}
if (-not (Test-Path $installer)) { throw "Installer was not created: $installer" }

$checksum = Join-Path $root "dist\NPUtella-$version-SnapdragonXPlus-NPU.msi.sha256"
$hash = (Get-FileHash $installer -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $(Split-Path -Leaf $installer)" | Set-Content -Path $checksum -Encoding ascii
$manifest = Join-Path $root "dist\stage\manifest.json"
if (-not (Test-Path $manifest)) { throw "Manifest was not created: $manifest" }

$notes = @"
NPUtella $version

Local NPU dictation for Snapdragon X Plus Windows ARM64 devices.

Includes English, French, and bilingual EN/FR auto-detection today.
More languages are planned for later releases.

No Python, Qualcomm AI Hub token, or first-run model download is required.
"@

& $gh.Source release create $tag $installer $checksum $manifest --title "NPUtella $version" --notes $notes
