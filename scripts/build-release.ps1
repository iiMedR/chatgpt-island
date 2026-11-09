[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
$oldEncodedFlags=$env:CARGO_ENCODED_RUSTFLAGS
$pathFlags=@("--remap-path-prefix=$projectRoot=.","--remap-path-prefix=$env:USERPROFILE=USERPROFILE") -join [char]31
$env:CARGO_ENCODED_RUSTFLAGS=if ($oldEncodedFlags) { $oldEncodedFlags+[char]31+$pathFlags } else { $pathFlags }
Push-Location $projectRoot
try {
    & node scripts/check-release.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Release preflight failed.' }
    & npm.cmd run tauri -- build --ci --target x86_64-pc-windows-msvc --bundles nsis
    if ($LASTEXITCODE -ne 0) { throw 'Production build failed.' }
    & node scripts/check-release.mjs --frontend
    if ($LASTEXITCODE -ne 0) { throw 'Production frontend audit failed.' }
    $bundle=Join-Path $projectRoot 'src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis'
    $installers=@(Get-ChildItem -LiteralPath $bundle -Filter '*-setup.exe' -File)
    if ($installers.Count -ne 1) { throw 'Expected exactly one NSIS installer. Remove stale build outputs before retrying.' }
    $output=Join-Path $projectRoot 'artifacts/release'
    New-Item -ItemType Directory -Force -Path $output | Out-Null
    $asset='ChatGPT-Island-Windows-x64-Setup.exe'
    $destination=Join-Path $output $asset
    Copy-Item -LiteralPath $installers[0].FullName -Destination $destination -Force
    $digest=(Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText((Join-Path $output 'SHA256SUMS.txt'),"$digest  $asset`n",[Text.UTF8Encoding]::new($false))
    $version=(Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version
    $report=[ordered]@{version=$version;target='x86_64-pc-windows-msvc';asset=$asset;bytes=(Get-Item -LiteralPath $destination).Length;sha256=$digest;authenticode=(Get-AuthenticodeSignature -LiteralPath $destination).Status.ToString();installerExecutionTested=$false}
    [IO.File]::WriteAllText((Join-Path $output 'release-manifest.json'),($report | ConvertTo-Json)+"`n",[Text.UTF8Encoding]::new($false))
    & node scripts/audit-package.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Package audit failed.' }
    $report | ConvertTo-Json
} finally { Pop-Location; $env:CARGO_ENCODED_RUSTFLAGS=$oldEncodedFlags }
