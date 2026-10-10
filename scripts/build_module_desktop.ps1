# Build the CPPlayer **QQ Music** JNI provider module (Windows desktop).
#
# Requires the MSVC toolchain (Visual Studio or Build Tools with the
# "Desktop development with C++" workload) plus the Windows SDK.
# The script locates them via vswhere / the standard Windows Kits path and
# sets PATH/INCLUDE/LIB itself, so no "Developer Command Prompt" is needed.
#
# Output: target\cp_qq_module_windows_<arch>.zip
#   Import it in the app via Settings -> Providers -> Import module.
#
# The desktop package is single-arch and its layout is:
#   manifest.json
#   netease_qq_music_api.dll
# (the library sits at the package root; supportedAbis is dropped from the manifest)

[CmdletBinding()]
param(
    # e.g. "E:\project\build\VC\Tools\MSVC" (the directory that contains version folders)
    [string]$MsvcRoot,
    [string]$SdkRoot = "${env:ProgramFiles(x86)}\Windows Kits\10"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Get-LatestVersionDir {
    param([string]$Parent, [string]$Probe)
    if (-not (Test-Path $Parent)) { return $null }
    Get-ChildItem -Directory $Parent |
        Sort-Object { try { [version]$_.Name } catch { [version]'0.0.0' } } -Descending |
        Where-Object { Test-Path (Join-Path $_.FullName $Probe) } |
        Select-Object -First 1 -ExpandProperty FullName
}

# ---- locate MSVC ---------------------------------------------------------
if (-not $MsvcRoot) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $install = & $vswhere -latest -products * -property installationPath | Select-Object -First 1
        if ($install) { $MsvcRoot = Join-Path $install "VC\Tools\MSVC" }
    }
}
if (-not $MsvcRoot -or -not (Test-Path $MsvcRoot)) {
    throw "MSVC not found. Install Visual Studio Build Tools (C++ workload) or pass -MsvcRoot <...\VC\Tools\MSVC>."
}

$msvcVersionDir = Get-LatestVersionDir -Parent $MsvcRoot -Probe "bin\Hostx64\x64\cl.exe"
if (-not $msvcVersionDir) { throw "cl.exe not found under $MsvcRoot." }

# ---- locate Windows SDK --------------------------------------------------
$sdkVersionDir = Get-LatestVersionDir -Parent (Join-Path $SdkRoot "Lib") -Probe "um\x64\kernel32.lib"
if (-not $sdkVersionDir) { throw "Windows SDK libs not found under $SdkRoot." }
$sdkVersion = Split-Path -Leaf $sdkVersionDir

# ---- export the toolchain environment -----------------------------------
$env:Path = "$(Join-Path $msvcVersionDir 'bin\Hostx64\x64');" + $env:Path
$env:INCLUDE = (@(
        (Join-Path $msvcVersionDir 'include'),
        (Join-Path $SdkRoot "Include\$sdkVersion\ucrt"),
        (Join-Path $SdkRoot "Include\$sdkVersion\um"),
        (Join-Path $SdkRoot "Include\$sdkVersion\shared")
    ) -join ';')
$env:LIB = (@(
        (Join-Path $msvcVersionDir 'lib\x64'),
        (Join-Path $SdkRoot "Lib\$sdkVersion\ucrt\x64"),
        (Join-Path $SdkRoot "Lib\$sdkVersion\um\x64")
    ) -join ';')

Write-Host "==> MSVC  : $msvcVersionDir"
Write-Host "==> SDK   : $SdkRoot ($sdkVersion)"

# ---- build ---------------------------------------------------------------
Write-Host "==> cargo build --release --lib --features jni"
# cargo writes progress to stderr; with ErrorActionPreference=Stop that would be
# turned into a terminating error, so relax it for the native call and check the
# exit code explicitly.
$previousEap = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
& cargo build --release --lib --features jni 2>&1 | Out-Host
$buildExit = $LASTEXITCODE
$ErrorActionPreference = $previousEap
if ($buildExit -ne 0) { throw "cargo build failed with exit code $buildExit." }

# ---- package -------------------------------------------------------------
$libName = "netease_qq_music_api.dll"
# Platform-specific staging dir with a fixed layout (manifest.json + one dll):
# files are simply overwritten, so no recursive delete is needed (that also
# avoids clashing with antivirus / safe-delete tooling).
$outDir = Join-Path $Root "target\cp_qq_module_windows"
$zipPath = Join-Path $Root "target\cp_qq_module_windows_x64.zip"

[System.IO.Directory]::CreateDirectory($outDir) | Out-Null
Copy-Item (Join-Path $Root "target\release\$libName") (Join-Path $outDir $libName) -Force

# Rewrite the manifest: desktop entryPoint + drop supportedAbis.
# Read/write as UTF-8 WITHOUT BOM (a BOM would break the host's JSON parser).
$manifest = [System.IO.File]::ReadAllText(
    (Join-Path $Root "packaging\manifest.json"), [System.Text.Encoding]::UTF8) | ConvertFrom-Json
$manifest.entryPoint = $libName
$manifest.PSObject.Properties.Remove("supportedAbis")
$json = $manifest | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText(
    (Join-Path $outDir "manifest.json"), $json, (New-Object System.Text.UTF8Encoding($false)))

# Zip with forward-slash entry names (portable across unzip implementations).
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
if ([System.IO.File]::Exists($zipPath)) { [System.IO.File]::Delete($zipPath) }
$archive = [System.IO.Compression.ZipFile]::Open($zipPath, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    Get-ChildItem -Recurse -File $outDir | ForEach-Object {
        $relative = $_.FullName.Substring($outDir.Length + 1).Replace('\', '/')
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
            $archive, $_.FullName, $relative) | Out-Null
    }
}
finally { $archive.Dispose() }

Write-Host "==> done: $zipPath"
