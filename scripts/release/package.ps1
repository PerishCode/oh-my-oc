$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))
$appDir = Join-Path $root 'app'
$name = 'oh-my-oc'
$cargoToml = Join-Path $appDir 'Cargo.toml'
$version = (Select-String -Path $cargoToml -Pattern '^version = "(.+)"$').Matches[0].Groups[1].Value
$releaseVersion = if ($args.Length -gt 0 -and -not [string]::IsNullOrWhiteSpace($args[0])) { $args[0] } elseif ($env:RELEASE_VERSION) { $env:RELEASE_VERSION } else { $version }
$target = if ($env:TARGET) { $env:TARGET } else { 'x86_64-pc-windows-msvc' }
$distDir = if ($env:DIST_DIR) { $env:DIST_DIR } else { Join-Path $root 'dist' }
$artifactDir = Join-Path $distDir $releaseVersion

New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

cargo build --release --manifest-path $cargoToml --target $target

$archive = "$name-$target.zip"
$skillZip = 'skill.zip'
$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("$name-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $bin = Join-Path $appDir "target/$target/release/$name.exe"
    Copy-Item $bin (Join-Path $tmpdir "$name.exe")
    Compress-Archive -LiteralPath (Join-Path $tmpdir "$name.exe") -DestinationPath (Join-Path $artifactDir $archive) -Force
    New-Item -ItemType Directory -Force -Path (Join-Path $tmpdir 'oh-my-oc') | Out-Null
    Copy-Item (Join-Path $root 'artifacts/skill/oh-my-oc/SKILL.md') (Join-Path $tmpdir 'oh-my-oc/SKILL.md')
    Compress-Archive -LiteralPath (Join-Path $tmpdir 'oh-my-oc') -DestinationPath (Join-Path $artifactDir $skillZip) -Force
    $hash = (Get-FileHash -Algorithm SHA256 -Path (Join-Path $artifactDir $archive)).Hash.ToLowerInvariant()
    Set-Content -Path (Join-Path $artifactDir 'checksums.txt') -Value @(
        "VERSION: $releaseVersion"
        "$hash  $archive"
        ((Get-FileHash -Algorithm SHA256 -Path (Join-Path $artifactDir $skillZip)).Hash.ToLowerInvariant() + "  $skillZip")
    )

    Write-Output (Join-Path $artifactDir $archive)
    Write-Output (Join-Path $artifactDir $skillZip)
    Write-Output (Join-Path $artifactDir 'checksums.txt')
}
finally {
    Remove-Item -LiteralPath $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
}
