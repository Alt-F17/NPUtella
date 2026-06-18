$ErrorActionPreference = "Stop"

$root = [System.IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$version = (Get-Content -Path (Join-Path $root "Cargo.toml") | Select-String -Pattern '^version\s*=\s*"(.*?)"' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Select-Object -First 1)
if (-not $version) { throw "Could not read version from Cargo.toml" }
if ($version -notmatch '^\d+\.\d+\.\d+$') {
    throw "MSI Package Version must be numeric major.minor.patch; Cargo.toml has '$version'"
}

$stage = Join-Path $root "dist\stage"
$installerDir = Join-Path $root "dist"
$generatedWxs = Join-Path $root "dist\generated-payload.wxs"
$msiPath = Join-Path $installerDir "NPUtella-$version-SnapdragonXPlus-NPU.msi"
$releaseAssetsRoot = Join-Path $root "release-assets"
$rustTarget = "aarch64-pc-windows-msvc"
$releaseExe = Join-Path $root "target\$rustTarget\release\nputella.exe"
$runtimeDst = Join-Path $stage "runtime\onnxruntime\capi"
$modelDst = Join-Path $stage "models\whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core"
$tokenizerDst = Join-Path $stage "whisper-base-local"

function Resolve-AssetPath([string[]]$candidates, [string]$label) {
    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path $candidate)) {
            return $candidate
        }
    }
    throw "Missing $label. Checked: $($candidates -join ', ')"
}

function Copy-RequiredFile([string]$sourceDir, [string]$fileName, [string]$destinationDir) {
    $source = Join-Path $sourceDir $fileName
    if (-not (Test-Path $source -PathType Leaf)) {
        throw "Missing required runtime file: $source"
    }
    Copy-Item -Force $source $destinationDir
}

function Copy-OptionalPattern([string]$sourceDir, [string]$pattern, [string]$destinationDir) {
    Get-ChildItem -Path $sourceDir -Filter $pattern -File -ErrorAction SilentlyContinue | ForEach-Object {
        Copy-Item -Force $_.FullName $destinationDir
    }
}

function Get-PeMachine([string]$path) {
    $bytes = [System.IO.File]::ReadAllBytes($path)
    if ($bytes.Length -lt 64) { throw "$path is too small to be a PE file" }
    if ($bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) { throw "$path is not a PE file" }
    $peOffset = [System.BitConverter]::ToInt32($bytes, 0x3c)
    if ($bytes.Length -lt ($peOffset + 6)) { throw "$path is missing PE headers" }
    if ($bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset + 1] -ne 0x45 -or $bytes[$peOffset + 2] -ne 0 -or $bytes[$peOffset + 3] -ne 0) {
        throw "$path has an invalid PE signature"
    }
    [System.BitConverter]::ToUInt16($bytes, $peOffset + 4)
}

function Escape-WixAttribute([string]$value) {
    [System.Security.SecurityElement]::Escape($value)
}

function Assert-RepoChildPath([string]$path, [string]$expectedLeaf, [string]$expectedParent) {
    $fullPath = [System.IO.Path]::GetFullPath($path)
    $leaf = Split-Path -Leaf $fullPath
    $parent = [System.IO.Path]::GetFullPath((Split-Path -Parent $fullPath))
    $expectedParentPath = [System.IO.Path]::GetFullPath((Join-Path $root $expectedParent))
    if ($leaf -ne $expectedLeaf -or $parent -ne $expectedParentPath) {
        throw "Refusing to modify unexpected path: $fullPath"
    }
}

$assetRoot = $env:NPUTELLA_RELEASE_ASSET_ROOT

$runtimeSrc = Resolve-AssetPath @(
    $(if ($assetRoot) { Join-Path $assetRoot "runtime\onnxruntime\capi" }),
    (Join-Path $releaseAssetsRoot "runtime\onnxruntime\capi")
) "runtime assets"

$modelSrc = Resolve-AssetPath @(
    $(if ($assetRoot) { Join-Path $assetRoot "models\whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core" }),
    (Join-Path $releaseAssetsRoot "models\whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core")
) "model assets"

$tokenizerSrc = Resolve-AssetPath @(
    $(if ($assetRoot) { Join-Path $assetRoot "whisper-base-local" }),
    (Join-Path $releaseAssetsRoot "whisper-base-local")
) "tokenizer assets"

cargo build --release --target $rustTarget
if ($LASTEXITCODE -ne 0) { throw "cargo build --release --target $rustTarget failed with exit code $LASTEXITCODE" }
$exePath = $releaseExe
$exeMachine = Get-PeMachine $exePath
if ($exeMachine -ne 0xAA64) {
    throw "$exePath is not ARM64 (machine=0x$($exeMachine.ToString('x4')))."
}

Assert-RepoChildPath $stage "stage" "dist"
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $runtimeDst | Out-Null
New-Item -ItemType Directory -Force -Path $modelDst | Out-Null
New-Item -ItemType Directory -Force -Path $tokenizerDst | Out-Null

Copy-Item -Force $exePath (Join-Path $stage "nputella.exe")
@(
    "onnxruntime.dll",
    "onnxruntime_providers_qnn.dll",
    "onnxruntime_providers_shared.dll",
    "QnnHtp.dll",
    "QnnHtpPrepare.dll",
    "QnnSystem.dll"
) | ForEach-Object {
    Copy-RequiredFile $runtimeSrc $_ $runtimeDst
}
Copy-OptionalPattern $runtimeSrc "Qnn*.dll" $runtimeDst
Copy-OptionalPattern $runtimeSrc "libQnn*.so" $runtimeDst
Copy-OptionalPattern $runtimeSrc "libqnn*.cat" $runtimeDst
Copy-Item -Recurse -Force (Join-Path $modelSrc "*") $modelDst
Copy-Item -Recurse -Force (Join-Path $tokenizerSrc "*") $tokenizerDst
Copy-Item -Force (Join-Path $root "start_nputella.vbs") (Join-Path $stage "start_nputella.vbs")
Copy-Item -Force (Join-Path $root "assets\nputella.ico") (Join-Path $stage "nputella.ico")

Push-Location $stage
try {
    & (Join-Path $stage "nputella.exe") --self-check
    if ($LASTEXITCODE -ne 0) { throw "nputella self-check failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}

$payloadFiles = Get-ChildItem -Path $stage -Recurse -File | Sort-Object FullName

$manifest = [ordered]@{
    version = $version
    device = "Snapdragon X Plus NPU Windows ARM64"
    languages = @("en", "fr", "bi")
    files = @()
}
$payloadFiles | ForEach-Object {
    $manifest.files += [ordered]@{
        path = $_.FullName.Substring($stage.Length + 1).Replace('\', '/')
        size = $_.Length
    }
}
$manifestPath = Join-Path $stage "manifest.json"
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $manifestPath -Encoding utf8

$checksumPath = Join-Path $stage "SHA256SUMS.txt"
$payloadFiles | ForEach-Object {
    $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $($_.FullName.Substring($stage.Length + 1).Replace('\', '/'))"
} | Set-Content -Path $checksumPath -Encoding ascii

function Convert-ToWixId([string]$value) {
    $id = $value -replace '[^A-Za-z0-9_\.]', '_'
    if ($id -notmatch '^[A-Za-z_]') { $id = "_$id" }
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($value)
    $hashBytes = $sha.ComputeHash($bytes)
    $hash = (($hashBytes | Select-Object -First 4 | ForEach-Object { $_.ToString("x2") }) -join "")
    if ($id.Length -gt 48) { $id = $id.Substring(0, 48) }
    $id = "$id`_$hash"
    return $id
}

$directories = @{}
$components = New-Object System.Collections.Generic.List[string]
$rootDirId = "INSTALLFOLDER"

$payloadFiles | ForEach-Object {
    $relative = $_.FullName.Substring($stage.Length + 1)
    $relativeDir = Split-Path $relative -Parent
    $dirId = $rootDirId
    if ($relativeDir) {
        $parts = $relativeDir -split '[\\/]'
        $pathSoFar = ""
        $parentId = $rootDirId
        foreach ($part in $parts) {
            $pathSoFar = if ($pathSoFar) { Join-Path $pathSoFar $part } else { $part }
            if (-not $directories.ContainsKey($pathSoFar)) {
                $directories[$pathSoFar] = [ordered]@{
                    id = "Dir_" + (Convert-ToWixId $pathSoFar)
                    name = $part
                    parent = $parentId
                }
            }
            $parentId = $directories[$pathSoFar].id
        }
        $dirId = $parentId
    }
    $componentId = "Cmp_" + (Convert-ToWixId $relative)
    $fileId = "Fil_" + (Convert-ToWixId $relative)
    $source = $_.FullName
    $components.Add("      <Component Id=""$componentId"" Directory=""$dirId"" Guid=""*""><File Id=""$fileId"" Source=""$(Escape-WixAttribute $source)"" KeyPath=""yes"" /></Component>")
}

$directoryXml = New-Object System.Collections.Generic.List[string]
foreach ($item in $directories.GetEnumerator() | Sort-Object { $_.Key.Length }) {
    $directoryXml.Add("    <DirectoryRef Id=""$($item.Value.parent)""><Directory Id=""$($item.Value.id)"" Name=""$(Escape-WixAttribute $item.Value.name)"" /></DirectoryRef>")
}

$payloadXml = @"
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Fragment>
$($directoryXml -join "`r`n")
  </Fragment>
  <Fragment>
    <ComponentGroup Id="AppPayload">
$($components -join "`r`n")
    </ComponentGroup>
  </Fragment>
</Wix>
"@
$payloadXml | Set-Content -Path $generatedWxs -Encoding utf8

$wix = Get-Command wix.exe -ErrorAction SilentlyContinue
if (-not $wix) { throw "wix.exe not found. Install WiX Toolset v4 or add wix.exe to PATH." }

& $wix.Source build (Join-Path $root "installer\nputella.wxs") $generatedWxs -define "AppVersion=$version" -define "SourceDir=$stage" -arch arm64 -out $msiPath
if ($LASTEXITCODE -ne 0) { throw "wix build failed with exit code $LASTEXITCODE" }

$msiHash = (Get-FileHash $msiPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$msiHash  $(Split-Path -Leaf $msiPath)" | Set-Content -Path "$msiPath.sha256" -Encoding ascii
