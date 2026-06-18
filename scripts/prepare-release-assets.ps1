param(
    [string]$AssetRoot = $env:NPUTELLA_RELEASE_ASSET_ROOT
)

$ErrorActionPreference = "Stop"

$root = [System.IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$bundleRoot = Join-Path $root "release-assets"

function Assert-RepoChildPath([string]$path, [string]$expectedName) {
    $fullPath = [System.IO.Path]::GetFullPath($path)
    $parent = Split-Path -Parent $fullPath
    $name = Split-Path -Leaf $fullPath
    if ($parent -ne $root -or $name -ne $expectedName) {
        throw "Refusing to modify unexpected path: $fullPath"
    }
}

function Resolve-AssetPath([string[]]$candidates, [string]$label) {
    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path $candidate)) {
            return $candidate
        }
    }
    throw "Missing $label. Checked: $($candidates -join ', ')"
}

if (-not $AssetRoot) {
    throw "NPUTELLA_RELEASE_ASSET_ROOT is required for release packaging. Point it at a prebuilt ARM64 asset root."
}
if (-not (Test-Path $AssetRoot)) {
    throw "NPUTELLA_RELEASE_ASSET_ROOT does not exist: $AssetRoot"
}

$runtimeSrc = Resolve-AssetPath @(
    (Join-Path $AssetRoot "runtime\onnxruntime\capi")
) "ARM64 runtime assets"

$modelSrc = Resolve-AssetPath @(
    (Join-Path $AssetRoot "models\whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core")
) "prebuilt model assets"

$tokenizerSrc = Resolve-AssetPath @(
    (Join-Path $AssetRoot "whisper-base-local"),
    (Join-Path $root "whisper-base-local")
) "tokenizer assets"

if (Test-Path $bundleRoot) {
    Assert-RepoChildPath $bundleRoot "release-assets"
    Remove-Item -Recurse -Force $bundleRoot
}
New-Item -ItemType Directory -Force -Path (Join-Path $bundleRoot "runtime\onnxruntime\capi") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $bundleRoot "models\whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $bundleRoot "whisper-base-local") | Out-Null

Copy-Item -Force (Join-Path $runtimeSrc "*") (Join-Path $bundleRoot "runtime\onnxruntime\capi")
Copy-Item -Recurse -Force (Join-Path $modelSrc "*") (Join-Path $bundleRoot "models\whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core")
Copy-Item -Recurse -Force (Join-Path $tokenizerSrc "*") (Join-Path $bundleRoot "whisper-base-local")
