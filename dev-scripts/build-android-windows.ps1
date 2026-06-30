# Best-effort Android build on Windows (NOT officially supported — see dev-docs/building.md).
# Official CI builds Android on Linux. Use WSL Ubuntu if possible.
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$Junction = "C:\Users\HASAN\touchHLE-build\touchHLE-trunk"
if (-not (Test-Path $Junction)) {
    cmd /c mklink /J "$Junction" "$Root"
}

& "$Root\dev-scripts\prepare-android-assets.ps1"

$ninjaDir = "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\Ninja-build.Ninja_Microsoft.Winget.Source_8wekyb3d8bbwe"
$env:JAVA_HOME = "C:\Program Files\Microsoft\jdk-17.0.19.10-hotspot"
$env:PATH = "$env:JAVA_HOME\bin;$ninjaDir;$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:ANDROID_SDK_ROOT = "C:\Users\HASAN\Android\Sdk"
$env:ANDROID_HOME = $env:ANDROID_SDK_ROOT
$env:ANDROID_NDK_ROOT = "$env:ANDROID_SDK_ROOT\ndk\25.2.9519653"
$env:ANDROID_NDK_HOME = $env:ANDROID_NDK_ROOT
$env:ANDROID_NDK = $env:ANDROID_NDK_ROOT
$env:CMAKE = "cmake"
$env:CMAKE_POLICY_VERSION_MINIMUM = "3.5"
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue

$gradle = "C:\Users\HASAN\touchHLE-build\gradle-8.11.1\bin\gradle.bat"
Push-Location "$Junction\android"
try {
    & $gradle clean assembleRelease --no-daemon
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "APK: $Junction\android\app\build\outputs\apk\release\app-release.apk"
