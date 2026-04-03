$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))
$version = if ($args.Length -gt 0) { $args[0] } else { '' }
$repo = if ($env:GITHUB_REPOSITORY) { $env:GITHUB_REPOSITORY } else { 'PerishCode/oh-my-oc' }

if ([string]::IsNullOrWhiteSpace($version)) { throw 'missing release version' }

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("oh-my-oc-smoke-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $env:HOME = Join-Path $tmpdir 'home'
    $env:OH_MY_OC_INSTALL_ROOT = Join-Path $tmpdir 'install'
    $env:OH_MY_OC_LOCAL_BIN_DIR = Join-Path $tmpdir 'bin'
    New-Item -ItemType Directory -Force -Path $env:HOME,$env:OH_MY_OC_INSTALL_ROOT,$env:OH_MY_OC_LOCAL_BIN_DIR | Out-Null

    & (Join-Path $root 'scripts/manage/omo.ps1') install --version $version

    if (-not (Test-Path -LiteralPath (Join-Path $env:OH_MY_OC_INSTALL_ROOT "$version/oh-my-oc.exe") -PathType Leaf)) { throw 'missing installed binary' }
    if (-not (Test-Path -LiteralPath (Join-Path $env:OH_MY_OC_LOCAL_BIN_DIR 'oh-my-oc.exe') -PathType Leaf)) { throw 'missing local bin copy' }

    $skillsDir = Join-Path $env:HOME '.agents\skills'
    New-Item -ItemType Directory -Force -Path $skillsDir | Out-Null
    $skillZipPath = Join-Path $tmpdir 'skill.zip'
    Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/$repo/releases/download/$version/skill.zip" -OutFile $skillZipPath
    Expand-Archive -LiteralPath $skillZipPath -DestinationPath $skillsDir -Force
    if (-not (Test-Path -LiteralPath (Join-Path $skillsDir 'oh-my-oc/SKILL.md') -PathType Leaf)) { throw 'missing skill asset' }

    $target = Join-Path $tmpdir 'target'
    New-Item -ItemType Directory -Force -Path $target | Out-Null
    & (Join-Path $env:OH_MY_OC_LOCAL_BIN_DIR 'oh-my-oc.exe') patch --path $target

    if (-not (Test-Path -LiteralPath (Join-Path $target 'opencode.json') -PathType Leaf)) { throw 'missing opencode.json' }
    if (-not (Test-Path -LiteralPath (Join-Path $target 'agent/commander.md') -PathType Leaf)) { throw 'missing agent/commander.md' }
}
finally {
    Remove-Item -LiteralPath $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
}
