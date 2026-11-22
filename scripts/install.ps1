# Vythea for Windows.
#
#   iex (irm https://vythea.com/install.ps1)
#
# This only delivers the program. It downloads the latest release into a
# temporary folder and hands over to `vythea setup`, which asks your name,
# signs you in, picks a provider and a model, and puts the window and the
# terminal where you choose. Nothing needs administrator rights, and nothing
# goes outside your own account.
#
# VYTHEA_DOWNLOAD  a different archive, as a URL or a local path
# VYTHEA_NO_SETUP  set to anything to unpack and stop, for unattended use

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$ProgressPreference = 'SilentlyContinue'   # the progress bar makes downloads several times slower

$source = if ($env:VYTHEA_DOWNLOAD) {
    $env:VYTHEA_DOWNLOAD
} else {
    'https://github.com/Vythea-ai/vythea-app/releases/latest/download/vythea-windows-x64.zip'
}

Write-Host ''
Write-Host '  Vythea' -ForegroundColor Blue -NoNewline
Write-Host '  installer' -ForegroundColor DarkGray
Write-Host ''

# Run a slow step in the background and turn a wheel beside it, so a quiet
# download does not look like a stalled one.
function Busy([string] $label, [string] $done, [scriptblock] $work, [object[]] $with) {
    $job = Start-Job -ScriptBlock $work -ArgumentList $with
    $frames = [char[]]'⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $i = 0
    while ($job.State -eq 'Running') {
        $seconds = [int]$clock.Elapsed.TotalSeconds
        $time = if ($seconds -gt 0) { " ${seconds}s" } else { '' }
        Write-Host -NoNewline ("`r  " + $frames[$i % $frames.Length] + " $label$time   ")
        $i++
        Start-Sleep -Milliseconds 90
    }
    Write-Host -NoNewline ("`r" + (' ' * ($label.Length + 16)) + "`r")
    try {
        Receive-Job $job -ErrorAction Stop | Out-Null
    } finally {
        Remove-Job $job -Force
    }
    Write-Host '  ✓ ' -ForegroundColor Blue -NoNewline
    Write-Host $done -ForegroundColor DarkGray
}

$staging = Join-Path ([IO.Path]::GetTempPath()) ("vythea-" + [Guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Path $staging | Out-Null
$archive = Join-Path $staging 'vythea.zip'

try {
    if (Test-Path $source) {
        Copy-Item $source $archive
        Write-Host '  ✓ ' -ForegroundColor Blue -NoNewline
        Write-Host 'using a local release' -ForegroundColor DarkGray
    } else {
        Busy 'downloading the latest release' 'downloaded' {
            param($from, $to)
            $ProgressPreference = 'SilentlyContinue'
            Invoke-WebRequest -Uri $from -OutFile $to -UseBasicParsing
        } @($source, $archive)
    }
} catch {
    Write-Host "  Could not download $source" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor DarkGray
    exit 1
}

Busy 'unpacking' 'unpacked' {
    param($from, $to)
    $ProgressPreference = 'SilentlyContinue'
    Expand-Archive -Path $from -DestinationPath $to -Force
} @($archive, $staging)
$exe = Get-ChildItem -Path $staging -Recurse -Filter 'vythea.exe' | Select-Object -First 1
if (-not $exe) {
    Write-Host '  The download did not contain vythea.exe.' -ForegroundColor Red
    exit 1
}

if ($env:VYTHEA_NO_SETUP) {
    Write-Host "  Unpacked to $($exe.DirectoryName)" -ForegroundColor DarkGray
    exit 0
}

# Setup asks questions, so it runs in this console, in the foreground. It
# copies itself into place, so the temporary copy can go when it is done.
& $exe.FullName setup
$code = $LASTEXITCODE
Remove-Item -Recurse -Force $staging -ErrorAction SilentlyContinue
exit $code
