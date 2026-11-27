<#
.SYNOPSIS
  VictoryXCL Two-Phase PR Security Gate & Hardware Benchmark Evaluator.

.DESCRIPTION
  Phase 1 (ALWAYS runs first, NEVER compiles or executes PR code):
    - Inspects PR diff (from -PrNumber via gh, or -Ref local branch/commit) against -BaseRef.
    - Enforces SECURITY-GATE.md blocklists on high-risk files (build.rs, Cargo.toml, workflows, scripts)
      and dangerous added patterns (process spawning, network calls, injection, persistence, test-skipping).
    - Prints the modified file list, diff stat, and any flagged lines, then STOPS unless -SecurityApproved is passed.

  Phase 2 (ONLY runs when -SecurityApproved is explicitly passed after Antigravity/human review):
    - Creates an isolated temporary git worktree (cleaned up in finally).
    - Runs unit tests (`cargo test --release -p xcl --lib -- --test-threads=1`).
    - Builds `VictoryXCL.exe` in release mode.
    - If -GameDir is provided, runs a timed baseline vs PR benchmark via scripts\smoke.ps1 and compares:
      crashes, device removals, frames shown, FPS, write-watch ms, and peak commit.
    - Outputs an objective APPROVE CANDIDATE / REJECT verdict (never auto-merges).

.EXAMPLE
  .\scripts\eval-pr.ps1 -PrNumber 1
  .\scripts\eval-pr.ps1 -PrNumber 1 -SecurityApproved -GameDir "C:\Path\To\Game" -Seconds 90
#>
param(
    [int]$PrNumber = 0,
    [string]$Ref = '',
    [string]$BaseRef = 'victoryxcl/main',
    [string]$Repo = 'CarlosB456/VictoryXCL',
    [switch]$SecurityApproved,
    [string]$GameDir = '',
    [int]$Seconds = 90,
    [string]$TestFilter = '',
    [switch]$SkipBenchmark
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path $PSScriptRoot -Parent

function Get-TargetRemote {
    $remotes = @(git -C $RepoRoot remote -v)
    foreach ($line in $remotes) {
        if ($line -match '^(\S+)\s+.*CarlosB456/VictoryXCL') {
            return $Matches[1]
        }
    }
    return 'origin'
}

function Invoke-GitQuiet {
    param([string[]]$GitArgs)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $null = & git -C $RepoRoot @GitArgs 2>&1
    } finally {
        $ErrorActionPreference = $prev
    }
}

$TargetRemote = Get-TargetRemote

function Resolve-TargetCommit {
    if ($PrNumber -gt 0) {
        $null = Get-Command gh -ErrorAction Stop
        Write-Host "[Phase 1] Fetching PR #$PrNumber metadata & ref from $Repo (no checkout, no build)..."
        $prJson = gh pr view $PrNumber --repo $Repo --json number,title,author,headRefOid,baseRefName,state | ConvertFrom-Json
        if (-not $prJson) { throw "Could not load PR #$PrNumber via gh" }
        Write-Host "  PR #$($prJson.number): $($prJson.title) by @$($prJson.author.login) (state: $($prJson.state), head: $($prJson.headRefOid))"
        Invoke-GitQuiet @('fetch', $TargetRemote, "pull/$PrNumber/head:refs/remotes/$TargetRemote/pr-$PrNumber")
        return [pscustomobject]@{
            Commit = (git -C $RepoRoot rev-parse "refs/remotes/$TargetRemote/pr-$PrNumber").Trim()
            Label  = "PR #$PrNumber (@$($prJson.author.login))"
        }
    }
    if (-not $Ref) {
        throw "Provide either -PrNumber <N> or -Ref <branch-or-sha>."
    }
    return [pscustomobject]@{
        Commit = (git -C $RepoRoot rev-parse $Ref).Trim()
        Label  = $Ref
    }
}

$target = Resolve-TargetCommit
Invoke-GitQuiet @('fetch', $TargetRemote, 'main')
$baseCommit = (git -C $RepoRoot rev-parse $BaseRef).Trim()
$diffRange = "$baseCommit...$($target.Commit)"

Write-Host ""
Write-Host "=== PHASE 1: STATIC SECURITY & ANTI-SLOP AUDIT ($($target.Label)) ==="
$changedFiles = @(git -C $RepoRoot diff --name-only $diffRange)
if ($changedFiles.Count -eq 0) {
    Write-Host "No changes between $BaseRef and $($target.Label)."
    exit 0
}

Write-Host "Changed files ($($changedFiles.Count)):"
git -C $RepoRoot diff --stat $diffRange | Write-Host

$highRiskPaths = @(
    '(^|/)build\.rs$',
    '(^|/)Cargo\.(toml|lock)$',
    '^\.github/',
    '^scripts/',
    '\.(ps1|bat|cmd|py|sh|vbs|js|exe|dll|pdb|bin|rpf)$'
)

$allowedScopeRoots = '^(crates/|tests/|docs/|README\.md$|TASKS\.(md|json)$|LEADERBOARD\.md$)'

$flaggedPaths = @()
$offTopicPaths = @()
foreach ($f in $changedFiles) {
    if ($f -notmatch $allowedScopeRoots) {
        $offTopicPaths += $f
    }
    foreach ($pat in $highRiskPaths) {
        if ($f -match $pat) {
            $flaggedPaths += $f
            break
        }
    }
}

$dangerPatterns = @(
    @{ Name = 'Process/Shell Spawn';   Regex = 'std::process::Command|Command::new|ShellExecute|CreateProcess|WinExec|powershell|cmd\.exe|wscript|mshta|rundll32|iex\s|EncodedCommand' },
    @{ Name = 'Network/Download';      Regex = 'std::net::|TcpStream|UdpSocket|TcpListener|reqwest|ureq|hyper|WinHttp|wininet|WSAStartup|Invoke-WebRequest|Invoke-RestMethod|curl\.exe|wget\s' },
    @{ Name = 'Memory/Hook Injection'; Regex = 'CreateRemoteThread|WriteProcessMemory|VirtualAllocEx|SetWindowsHookEx|NtUnmapViewOfSection' },
    @{ Name = 'Persistence/Registry';  Regex = 'RegSetValue|RegCreateKey|CurrentVersion\\Run|schtasks' },
    @{ Name = 'Obfuscation/Blob';      Regex = 'include_bytes!\(|from_base64|base64::decode|FromBase64String' },
    @{ Name = 'Prompt Injection';      Regex = 'ignore (all )?previous instructions|disregard (all )?(above|prior)|system prompt|developer mode|jailbreak|auto[- ]?approve|skip security' },
    @{ Name = 'Slop/Test Bypass';      Regex = '#\[ignore\]|XCL_NETWORK\s*=\s*1' }
)

$diffLines = @(git -C $RepoRoot diff --unified=0 $diffRange)
$currentFile = ''
$flaggedCode = @()

foreach ($line in $diffLines) {
    if ($line -match '^\+\+\+ b/(.+)$') {
        $currentFile = $Matches[1]
        continue
    }
    if ($line -match '^\+[^\+]') {
        $codeLine = $line.Substring(1)
        foreach ($rule in $dangerPatterns) {
            if ($codeLine -match $rule.Regex) {
                $flaggedCode += [pscustomobject]@{
                    Rule = $rule.Name
                    File = $currentFile
                    Line = $codeLine.Trim()
                }
            }
        }
    }
}

Write-Host ""
if ($offTopicPaths.Count -gt 0) {
    Write-Host "[SCOPE WARNING] Files outside VictoryXCL core scope (crates/, tests/, docs/, README/TASKS/LEADERBOARD):" -ForegroundColor Yellow
    $offTopicPaths | Sort-Object -Unique | ForEach-Object { Write-Host "  - $_" -ForegroundColor Yellow }
}
if ($flaggedPaths.Count -gt 0) {
    Write-Host "[SECURITY WARNING] High-risk paths modified:" -ForegroundColor Yellow
    $flaggedPaths | Sort-Object -Unique | ForEach-Object { Write-Host "  - $_" -ForegroundColor Yellow }
}
if ($flaggedCode.Count -gt 0) {
    Write-Host "[SECURITY WARNING] Suspicious added patterns detected:" -ForegroundColor Red
    $flaggedCode | ForEach-Object { Write-Host "  [$($_.Rule)] $($_.File): $($_.Line)" -ForegroundColor Red }
}
if ($offTopicPaths.Count -eq 0 -and $flaggedPaths.Count -eq 0 -and $flaggedCode.Count -eq 0) {
    Write-Host "[Phase 1] Automated scan found 0 off-topic paths, 0 blocklisted paths, and 0 dangerous added patterns." -ForegroundColor Green
}

if (-not $SecurityApproved) {
    Write-Host ""
    Write-Host "STOPPED BEFORE COMPILATION (Fail-Closed Security Gate)." -ForegroundColor Cyan
    Write-Host "Next steps:"
    Write-Host "  1. Review the full diff with Antigravity / git diff $diffRange against SECURITY-GATE.md"
    Write-Host "  2. If completely safe, re-run with -SecurityApproved to execute Phase 2 (tests + benchmark)."
    exit 2
}

Write-Host ""
Write-Host "=== PHASE 2: ISOLATED WORKTREE BUILD, TEST & BENCHMARK ==="

function Invoke-SmokeMetrics {
    param(
        [string]$ExePath,
        [string]$Label,
        [string]$OutSubDir
    )
    $smokeScript = Join-Path $RepoRoot 'scripts\smoke.ps1'
    if (-not (Test-Path $smokeScript)) { throw "Missing $smokeScript" }

    # Apply VictoryXCL hardware defaults for RX 6600 + 16 GB RAM without mutating parent shell permanently
    $savedEnv = @{}
    $benchEnv = @{
        XCL_DEVICE_TYPE         = 'lockhart'
        XCL_D3D12_DIR           = (Join-Path (Split-Path $RepoRoot -Parent | Split-Path -Parent) 'D3D12')
        XCL_RAYTRACING          = 'off'
        XCL_ADAPTER             = 'auto'
        XCL_TEXTURE_LEVEL_PAGES = '1'
        XCL_HEAP_WINDOWS        = '1'
        XCL_UMA_WINDOWS         = '8'
        XCL_PORT_CONSTANTS      = '1'
        XCL_WATCH_GAP_MB        = '64'
        XCL_WATCH_COALESCE_MS   = '12'
        XCL_COHERENT_MB         = '2048'
        XCL_COMMIT_RESERVE_MB   = '4096'
        XCL_OUTPUT              = '1920x1080'
        XCL_FRAME_INTERVAL_MS   = '33.333'
        XCL_PSO_DISK_CACHE      = '1'
        XCL_PSO_FRAME_BUDGET_MS = '40'
        XCL_PSO_FRAME_WAIT_MS   = '16'
        XCL_PSO_THREADS         = '4'
        XCL_DXC_THREADS         = '4'
        XCL_AMBIENT             = 'auto'
    }
    foreach ($k in $benchEnv.Keys) {
        $savedEnv[$k] = [Environment]::GetEnvironmentVariable($k, 'Process')
        [Environment]::SetEnvironmentVariable($k, $benchEnv[$k], 'Process')
    }
    try {
        & $smokeScript -Game $GameDir -Seconds $Seconds -Xcl $ExePath -Title $Label -OutDir $OutSubDir | Write-Host
    } finally {
        foreach ($k in $benchEnv.Keys) {
            [Environment]::SetEnvironmentVariable($k, $savedEnv[$k], 'Process')
        }
    }

    $manifest = Get-Content (Join-Path $OutSubDir 'manifest.json') -Raw | ConvertFrom-Json
    $logText = Get-Content (Join-Path $OutSubDir 'xcl.log') -Raw
    $fpsSamples = @([regex]::Matches($logText, '([0-9]+(?:\.[0-9]+)?) fps over the last 300') | ForEach-Object { [double]$_.Groups[1].Value })
    $avgFps = if ($fpsSamples.Count -gt 0) { [math]::Round(($fpsSamples | Measure-Object -Average).Average, 2) } else { 0.0 }
    $minFps = if ($fpsSamples.Count -gt 0) { [math]::Round(($fpsSamples | Measure-Object -Minimum).Minimum, 2) } else { 0.0 }
    $wwMatch = [regex]::Matches($logText, 'write-watch (\d+) ms') | Select-Object -Last 1
    $writeWatchMs = if ($wwMatch) { [int]$wwMatch.Groups[1].Value } else { -1 }

    return [pscustomobject]@{
        Label         = $Label
        Crashes       = [int]$manifest.crashes
        Removals      = [int]$manifest.removals
        PeakCommitMb  = $manifest.peak_commit_mb
        AvgFps        = $avgFps
        MinFps        = $minFps
        WriteWatchMs  = $writeWatchMs
    }
}

$wtDir = Join-Path $env:TEMP ("victoryxcl-eval-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
$benchDir = Join-Path $env:TEMP ("victoryxcl-bench-" + [guid]::NewGuid().ToString('N').Substring(0, 8))

try {
    Write-Host "Creating isolated worktree at $wtDir ($($target.Commit))..."
    Invoke-GitQuiet @('worktree', 'add', '--detach', $wtDir, $target.Commit)
    if ($LASTEXITCODE -ne 0) { throw "Failed to create git worktree" }

    Write-Host "Running unit tests in isolated worktree (with XCL_D3D12_DIR set)..."
    $d3d12Dir = Join-Path (Split-Path $RepoRoot -Parent | Split-Path -Parent) 'D3D12'
    $savedD3d12 = [Environment]::GetEnvironmentVariable('XCL_D3D12_DIR', 'Process')
    if (Test-Path $d3d12Dir) {
        [Environment]::SetEnvironmentVariable('XCL_D3D12_DIR', $d3d12Dir, 'Process')
    }
    $savedErr = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $testArgs = @('test', '--release', '--manifest-path', (Join-Path $wtDir 'Cargo.toml'), '-p', 'xcl', '--lib')
        if ($TestFilter) { $testArgs += @('--', $TestFilter, '--test-threads=1') } else { $testArgs += @('--', '--test-threads=1') }
        $prTestOut = & cargo @testArgs 2>&1 | Tee-Object -Variable prTestLines
        $prTestExit = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $savedErr
        [Environment]::SetEnvironmentVariable('XCL_D3D12_DIR', $savedD3d12, 'Process')
    }
    if ($prTestExit -ne 0) {
        $prFailed = @($prTestLines | Where-Object { $_ -match '^test (\S+) \.\.\. FAILED$' } | ForEach-Object { $Matches[1] })
        Write-Host "PR had $($prFailed.Count) failing test(s); checking against baseline ($baseCommit) for regressions..." -ForegroundColor Yellow
        $baseWt = Join-Path $env:TEMP ("victoryxcl-base-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
        try {
            Invoke-GitQuiet @('worktree', 'add', '--detach', $baseWt, $baseCommit)
            if (Test-Path $d3d12Dir) { [Environment]::SetEnvironmentVariable('XCL_D3D12_DIR', $d3d12Dir, 'Process') }
            $baseArgs = @('test', '--release', '--manifest-path', (Join-Path $baseWt 'Cargo.toml'), '-p', 'xcl', '--lib')
            if ($TestFilter) { $baseArgs += @('--', $TestFilter, '--test-threads=1') } else { $baseArgs += @('--', '--test-threads=1') }
            $baseTestLines = @(& cargo @baseArgs 2>&1)
            $baseFailed = @($baseTestLines | Where-Object { $_ -match '^test (\S+) \.\.\. FAILED$' } | ForEach-Object { $Matches[1] })
        } finally {
            [Environment]::SetEnvironmentVariable('XCL_D3D12_DIR', $savedD3d12, 'Process')
            if (Test-Path $baseWt) { Invoke-GitQuiet @('worktree', 'remove', '--force', $baseWt) }
        }
        $newFailures = @($prFailed | Where-Object { $_ -notin $baseFailed })
        if ($newFailures.Count -gt 0) {
            Write-Host "[VERDICT] REJECT: $($newFailures.Count) NEW failing test(s) introduced by PR:" -ForegroundColor Red
            $newFailures | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
            exit 1
        }
        Write-Host "0 new test regressions (all $($prFailed.Count) failures are pre-existing on $BaseRef)." -ForegroundColor Green
    }

    Write-Host "Building release VictoryXCL.exe in isolated worktree..."
    $savedErr2 = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & cargo build --release --manifest-path (Join-Path $wtDir 'Cargo.toml') --bin VictoryXCL 2>&1 | Out-Host
    $buildExit = $LASTEXITCODE
    $ErrorActionPreference = $savedErr2
    if ($buildExit -ne 0) {
        Write-Host "[VERDICT] REJECT: Release build failed." -ForegroundColor Red
        exit 1
    }

    if ($SkipBenchmark -or -not $GameDir) {
        Write-Host "[VERDICT] TESTS & BUILD PASSED (live benchmark skipped: pass -GameDir to benchmark against baseline)." -ForegroundColor Green
        exit 0
    }

    $baseExe = Join-Path $RepoRoot 'target\release\VictoryXCL.exe'
    $prExe = Join-Path $wtDir 'target\release\VictoryXCL.exe'
    if (-not (Test-Path $baseExe)) { throw "Baseline executable not found at $baseExe; build main first." }

    Write-Host "Running baseline benchmark ($Seconds s)..."
    $baseStats = Invoke-SmokeMetrics -ExePath $baseExe -Label 'baseline' -OutSubDir (Join-Path $benchDir 'baseline')
    Write-Host "Running PR benchmark ($Seconds s)..."
    $prStats = Invoke-SmokeMetrics -ExePath $prExe -Label 'pr' -OutSubDir (Join-Path $benchDir 'pr')

    Write-Host ""
    Write-Host "=== BENCHMARK COMPARISON ==="
    @($baseStats, $prStats) | Format-Table -AutoSize | Out-Host

    if ($prStats.Crashes -gt 0 -or $prStats.Removals -gt 0) {
        Write-Host "[VERDICT] REJECT: PR had $($prStats.Crashes) crash(es) and $($prStats.Removals) device removal(s)." -ForegroundColor Red
        exit 1
    }

    $fpsImproved = ($prStats.AvgFps -ge $baseStats.AvgFps)
    $wwImproved = ($prStats.WriteWatchMs -ge 0 -and $baseStats.WriteWatchMs -gt 0 -and $prStats.WriteWatchMs -lt $baseStats.WriteWatchMs)
    if ($prStats.AvgFps -lt ($baseStats.AvgFps * 0.95)) {
        Write-Host "[VERDICT] REJECT: FPS regressed (>5% below baseline: $($prStats.AvgFps) vs $($baseStats.AvgFps))." -ForegroundColor Red
        exit 1
    }
    if ($fpsImproved -or $wwImproved) {
        Write-Host "[VERDICT] APPROVE CANDIDATE: 0 crashes/removals and improved or equal performance." -ForegroundColor Green
        exit 0
    } else {
        Write-Host "[VERDICT] NEUTRAL / MANUAL REVIEW: 0 crashes, slight variance ($($prStats.AvgFps) vs $($baseStats.AvgFps) fps)." -ForegroundColor Yellow
        exit 0
    }
} finally {
    if (Test-Path $wtDir) {
        Invoke-GitQuiet @('worktree', 'remove', '--force', $wtDir)
    }
}
