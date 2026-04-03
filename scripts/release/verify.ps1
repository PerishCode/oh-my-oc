$ErrorActionPreference = 'Stop'

$mode = if ($args.Length -gt 0) { $args[0] } else { '' }
$releaseVersion = if ($args.Length -gt 1) { $args[1] } else { '' }
$artifactDir = if ($args.Length -gt 2) { $args[2] } else { '' }

if ([string]::IsNullOrWhiteSpace($mode)) { throw 'missing mode' }
if ([string]::IsNullOrWhiteSpace($releaseVersion)) { throw 'missing release version' }
if ([string]::IsNullOrWhiteSpace($artifactDir)) { throw 'missing artifact dir' }
if (-not (Test-Path -LiteralPath $artifactDir -PathType Container)) { throw "artifact dir missing: $artifactDir" }

function Require-File([string] $name) {
    $path = Join-Path $artifactDir $name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "missing artifact: $name"
    }
}

function Require-ChecksumEntry([string] $name) {
    $checksumsPath = Join-Path $artifactDir 'checksums.txt'
    foreach ($line in Get-Content -LiteralPath $checksumsPath) {
        $parts = $line -split '\s+'
        if ($parts.Length -ge 2 -and $parts[-1] -eq $name) {
            return
        }
    }

    throw "missing checksum entry: $name"
}

function Assert-ZipContains([string] $zipName, [string] $member) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zipPath = Join-Path $artifactDir $zipName
    $zip = [System.IO.Compression.ZipFile]::OpenRead($zipPath)
    try {
        if (-not ($zip.Entries.FullName -contains $member)) {
            throw "missing $member in $zipName"
        }
    }
    finally {
        $zip.Dispose()
    }
}

function Assert-TarContains([string] $tarName, [string] $member) {
    $tarPath = Join-Path $artifactDir $tarName
    $output = tar -tzf $tarPath
    if (-not ($output -contains $member)) {
        throw "missing $member in $tarName"
    }
}

switch ($mode) {
    'accept' {
        @(
            'checksums.txt',
            'oh-my-oc-x86_64-unknown-linux-gnu.tar.gz',
            'oh-my-oc-aarch64-apple-darwin.tar.gz',
            'oh-my-oc-x86_64-apple-darwin.tar.gz',
            'oh-my-oc-x86_64-pc-windows-msvc.zip',
            'skill.zip',
            'skill.tar.gz'
        ) | ForEach-Object { Require-File $_ }

        $versionLine = (Select-String -Path (Join-Path $artifactDir 'checksums.txt') -Pattern '^VERSION:\s*(.+)$').Matches[0].Groups[1].Value.Trim()
        if ($versionLine -ne $releaseVersion) {
            throw "version mismatch: expected $releaseVersion got $versionLine"
        }

        @(
            'oh-my-oc-x86_64-unknown-linux-gnu.tar.gz',
            'oh-my-oc-aarch64-apple-darwin.tar.gz',
            'oh-my-oc-x86_64-apple-darwin.tar.gz',
            'oh-my-oc-x86_64-pc-windows-msvc.zip',
            'skill.zip',
            'skill.tar.gz'
        ) | ForEach-Object { Require-ChecksumEntry $_ }
    }
    'verify' {
        Assert-TarContains 'oh-my-oc-x86_64-unknown-linux-gnu.tar.gz' 'oh-my-oc'
        Assert-TarContains 'oh-my-oc-aarch64-apple-darwin.tar.gz' 'oh-my-oc'
        Assert-TarContains 'oh-my-oc-x86_64-apple-darwin.tar.gz' 'oh-my-oc'
        Assert-ZipContains 'oh-my-oc-x86_64-pc-windows-msvc.zip' 'oh-my-oc.exe'
        Assert-TarContains 'skill.tar.gz' 'oh-my-oc/SKILL.md'
        Assert-ZipContains 'skill.zip' 'oh-my-oc/SKILL.md'
    }
    default {
        throw "unknown mode: $mode"
    }
}
