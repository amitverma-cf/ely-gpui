# Builds the browser gallery on Windows; ELY_GALLERY_ASSETS addresses <out>/assets/.
param (
    [Parameter(Mandatory=$true)]
    [string]$OutDir
)

$ErrorActionPreference = "Stop"

if (-not $env:ELY_GALLERY_ASSETS) {
    $env:ELY_GALLERY_ASSETS = "/gallery/assets/"
}

$Root = Resolve-Path "$PSScriptRoot\.."
Set-Location $Root

$OutResolved = New-Item -ItemType Directory -Force $OutDir | Select-Object -ExpandProperty FullName

$env:RUSTC_BOOTSTRAP = "1"
cargo build --example gallery --target wasm32-unknown-unknown --profile web

$Wasm = "$Root\target\wasm32-unknown-unknown\web\examples\gallery.wasm"
wasm-bindgen $Wasm --out-dir $OutResolved --out-name gallery --target web --no-typescript --remove-name-section --remove-producers-section

wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue "$OutResolved\gallery_bg.wasm" -o "$OutResolved\gallery_bg.wasm"

Copy-Item -Force "$Root\examples\gallery\index.html" "$OutResolved\"
Remove-Item -Recurse -Force "$OutResolved\assets" -ErrorAction SilentlyContinue
Copy-Item -Recurse -Force "$Root\examples\gallery\assets" "$OutResolved\assets"

$Size = (Get-Item "$OutResolved\gallery_bg.wasm").Length
Write-Host "gallery_bg.wasm: $Size bytes"
