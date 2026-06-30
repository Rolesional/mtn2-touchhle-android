# Windows: replace git symlinks in android/assets with real files (Linux/macOS use symlinks).
# Run from repo root before `gradle assembleRelease`.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Assets = Join-Path $Root "android\app\src\main\assets"

function Copy-AssetDir($Name) {
    $Src = Join-Path $Root $Name
    $Dst = Join-Path $Assets $Name
    if (-not (Test-Path $Src)) { throw "Missing: $Src" }
    if (Test-Path $Dst) { Remove-Item $Dst -Recurse -Force }
    Copy-Item $Src $Dst -Recurse -Force
    Write-Host "Copied $Name -> $Dst"
}

function Copy-AssetFile($Name) {
    $Src = Join-Path $Root $Name
    $Dst = Join-Path $Assets (Split-Path $Name -Leaf)
    if (-not (Test-Path $Src)) { throw "Missing: $Src" }
    Copy-Item $Src $Dst -Force
    Write-Host "Copied $Name -> $Dst"
}

Copy-AssetDir "touchHLE_fonts"
Copy-AssetDir "touchHLE_dylibs"
Copy-AssetFile "touchHLE_default_options.txt"

# MTN2 port assets
$Drawable = Join-Path $Root "android\app\src\main\res\drawable-nodpi"
New-Item -ItemType Directory -Force -Path $Drawable | Out-Null

$IconSrc = "C:\Users\HASAN\Downloads\6-1-150184-52(1).png"
$SplashSrc = "C:\Users\HASAN\Downloads\thumbnailmtn2.jpg"
$IpaSrc = Join-Path $Root "touchHLE_apps\MonsterTrucksNitroV120.ipa"

if (-not (Test-Path $IconSrc)) { throw "Missing launcher icon: $IconSrc" }
if (-not (Test-Path $SplashSrc)) { throw "Missing splash image: $SplashSrc" }
if (-not (Test-Path $IpaSrc)) { throw "Missing bundled IPA: $IpaSrc" }

Copy-Item $IconSrc (Join-Path $Drawable "icon.png") -Force
Write-Host "Copied launcher icon -> $Drawable\icon.png"
Copy-Item $SplashSrc (Join-Path $Drawable "splash_mtn2.jpg") -Force
Write-Host "Copied splash thumbnail -> $Drawable\splash_mtn2.jpg"
Copy-Item $IpaSrc (Join-Path $Assets "MonsterTrucksNitroV120.ipa") -Force
Write-Host "Copied bundled IPA -> $Assets\MonsterTrucksNitroV120.ipa"

$SandboxPlistSrc = "C:\Users\HASAN\Downloads\com.redlynx.MTN2.plist"
$SandboxPlistDst = Join-Path $Assets "sandbox\com.redlynx.MTN2\Library\Preferences\com.redlynx.MTN2.plist"
if (-not (Test-Path $SandboxPlistSrc)) { throw "Missing sandbox plist: $SandboxPlistSrc" }
New-Item -ItemType Directory -Force -Path (Split-Path $SandboxPlistDst) | Out-Null
Copy-Item $SandboxPlistSrc $SandboxPlistDst -Force
Write-Host "Copied sandbox prefs -> $SandboxPlistDst"

# SDL Java must match rust-sdl2's bundled SDL (libSDL2.so), not vendor/SDL submodule.
$SdlJavaSrc = Get-ChildItem "$env:USERPROFILE\.cargo\git\checkouts\rust-sdl2-*" -ErrorAction SilentlyContinue |
    ForEach-Object { Get-ChildItem $_.FullName -Recurse -Directory -Filter "java" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -like "*\android-project\app\src\main\java" } } |
    Select-Object -First 1
if (-not $SdlJavaSrc) {
    throw "Could not find rust-sdl2 SDL Java sources in cargo git checkouts. Run: cargo fetch"
}
$SdlJavaDst = Join-Path $Root "android\sdl-java"
if (Test-Path $SdlJavaDst) { Remove-Item $SdlJavaDst -Recurse -Force }
Copy-Item $SdlJavaSrc.FullName $SdlJavaDst -Recurse -Force
Write-Host "Copied SDL Java from $($SdlJavaSrc.FullName) -> $SdlJavaDst"

# compileSdk 31: Context.RECEIVER_EXPORTED not available
$Hid = Join-Path $SdlJavaDst "org\libsdl\app\HIDDeviceManager.java"
if (Test-Path $Hid) {
    (Get-Content $Hid -Raw) `
        -replace 'Context\.RECEIVER_EXPORTED', '0x00000002' |
        Set-Content $Hid -NoNewline
    Write-Host "Patched HIDDeviceManager for SDK 31"
}

Write-Host "Android assets ready."
