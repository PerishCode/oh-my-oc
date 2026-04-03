$ErrorActionPreference = 'Stop'

$name = if ($env:OH_MY_OC_NAME) { $env:OH_MY_OC_NAME } else { 'oh-my-oc' }
$repo = if ($env:OH_MY_OC_REPO) { $env:OH_MY_OC_REPO } else { 'PerishCode/oh-my-oc' }
$baseUrl = if ($env:OH_MY_OC_BASE_URL) { $env:OH_MY_OC_BASE_URL } else { "https://github.com/$repo/releases" }
$installRoot = if ($env:OH_MY_OC_INSTALL_ROOT) { $env:OH_MY_OC_INSTALL_ROOT } else { Join-Path $env:LOCALAPPDATA $name }
$localBinDir = if ($env:OH_MY_OC_LOCAL_BIN_DIR) { $env:OH_MY_OC_LOCAL_BIN_DIR } else { Join-Path $env:USERPROFILE '.local\bin' }
$command = if ($args.Length -gt 0) { $args[0] } else { 'install' }
$version = $env:OH_MY_OC_VERSION
$remainingArgs = @()

if ($args.Length -gt 1) {
    $remainingArgs = $args[1..($args.Length - 1)]
}

for ($i = 0; $i -lt $remainingArgs.Length; $i++) {
    switch ($remainingArgs[$i]) {
        '--version' {
            $i++
            if ($i -ge $remainingArgs.Length -or [string]::IsNullOrWhiteSpace($remainingArgs[$i])) {
                throw 'missing value for --version'
            }
            $version = $remainingArgs[$i]
        }
        default {
            throw "unknown argument: $($remainingArgs[$i])"
        }
    }
}

switch ($command) {
    'install' { }
    'upgrade' { }
    'uninstall' {
        Remove-Item -LiteralPath (Join-Path $localBinDir "$name.exe") -Force -ErrorAction SilentlyContinue
        if ($version) {
            Remove-Item -LiteralPath (Join-Path $installRoot $version) -Recurse -Force -ErrorAction SilentlyContinue
        } elseif (Test-Path $installRoot) {
            Remove-Item -LiteralPath $installRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
        return
    }
    default { throw "unknown command: $command" }
}

$target = switch ("$($env:PROCESSOR_ARCHITECTURE)") {
    'AMD64' { 'x86_64-pc-windows-msvc' }
    default { throw "unsupported host target: $($env:PROCESSOR_ARCHITECTURE)" }
}

$archive = "$name-$target.zip"
if ($version) {
    $releasePath = $version
    $checksumsUrl = "$baseUrl/download/$releasePath/checksums.txt"
    $archiveUrl = "$baseUrl/download/$releasePath/$archive"
} else {
    $releasePath = 'latest'
    $checksumsUrl = "$baseUrl/$releasePath/download/checksums.txt"
    $archiveUrl = "$baseUrl/$releasePath/download/$archive"
}

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("$name-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $checksumsPath = Join-Path $tmpdir 'checksums.txt'
    $archivePath = Join-Path $tmpdir $archive

    Invoke-WebRequest -UseBasicParsing -Uri $checksumsUrl -OutFile $checksumsPath
    Invoke-WebRequest -UseBasicParsing -Uri $archiveUrl -OutFile $archivePath

    if (-not $version) {
        $version = (Select-String -Path $checksumsPath -Pattern '^VERSION:\s*(.+)$').Matches[0].Groups[1].Value.Trim()
    }

    if ([string]::IsNullOrWhiteSpace($version)) {
        throw 'could not resolve release version'
    }

    $expected = $null
    foreach ($line in Get-Content $checksumsPath) {
        if ($line -match '^([0-9a-fA-F]{64})\s+\*?(.+)$' -and $matches[2] -eq $archive) {
            $expected = $matches[1].ToLowerInvariant()
            break
        }
    }

    if (-not $expected) {
        throw "artifact unavailable: $archive"
    }

    $actual = (Get-FileHash -Algorithm SHA256 -Path $archivePath).Hash.ToLowerInvariant()
    if ($expected -ne $actual) {
        throw 'checksum verification failed'
    }

    $installDir = Join-Path $installRoot $version
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    New-Item -ItemType Directory -Force -Path $localBinDir | Out-Null

    Expand-Archive -LiteralPath $archivePath -DestinationPath $tmpdir -Force
    Copy-Item -LiteralPath (Join-Path $tmpdir "$name.exe") -Destination (Join-Path $installDir "$name.exe") -Force
    Copy-Item -LiteralPath (Join-Path $installDir "$name.exe") -Destination (Join-Path $localBinDir "$name.exe") -Force

    Write-Output (Join-Path $localBinDir "$name.exe")
}
finally {
    Remove-Item -LiteralPath $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
}
