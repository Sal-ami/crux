#Requires -Version 5.1
param(
    [string]$CruxPath = "C:\Users\fetit\AppData\Local\Temp\opencode\crux-bench\crux.exe",
    [string]$Work = "C:\Users\fetit\AppData\Local\Temp\opencode\crux-bench"
)

$ErrorActionPreference = "SilentlyContinue"
$env:MSYS_NO_PATHCONV = "1"
$ReposDir = "$Work\repos"
$ResultsDir = "$Work\results"
if (-not (Test-Path $ReposDir)) { New-Item -ItemType Directory -Force -Path $ReposDir | Out-Null }
if (Test-Path $ResultsDir) { Remove-Item "$ResultsDir\*" -Force -Recurse } else { New-Item -ItemType Directory -Force -Path $ResultsDir | Out-Null }
$CruxPath = (Resolve-Path $CruxPath).Path

$script:T = [datetime]::Parse("2026-01-01 12:00:00")

function Set-BenchEnv {
    $env:GIT_AUTHOR_NAME="bench"; $env:GIT_AUTHOR_EMAIL="bench@b.dev"
    $env:GIT_COMMITTER_NAME="bench"; $env:GIT_COMMITTER_EMAIL="bench@b.dev"
}
Set-BenchEnv

function Clear-BenchEnv {
    Remove-Item Env:\GIT_AUTHOR_NAME -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_AUTHOR_EMAIL -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_NAME -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_EMAIL -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_AUTHOR_DATE -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_DATE -ErrorAction SilentlyContinue
}

function Commit {
    param([string]$Msg)
    $script:T = $script:T.AddMinutes(1)
    $d = $script:T.ToString("yyyy-MM-ddTHH:mm:ss")
    $env:GIT_AUTHOR_DATE=$d; $env:GIT_COMMITTER_DATE=$d
    git add -A 2>$null
    git commit -q -m $Msg 2>$null
}

function Init-Repo {
    param([string]$Name)
    $dir = "$ReposDir\$Name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    $script:T = [datetime]::Parse("2026-01-01 12:00:00")
    $script:Broken = $false
    git init -q 2>$null
    Set-Content ".gitignore" "test.cmd`n.execs`n.crux/" -Encoding ascii
    git add -A 2>$null
    git commit -q -m "initial" 2>$null
}

function Write-PassTest {
    @'
@echo off
echo x>>.execs
findstr /i "fail" status.txt >nul 2>&1
if %ERRORLEVEL%==0 (exit /b 1) else (exit /b 0)
'@ | Out-File "test.cmd" -Encoding ascii
}

function Write-LinearRepo {
    param([string]$Name, [int]$Total, [int]$BreakAt)
    Init-Repo $Name
    Write-PassTest
    for ($i = 1; $i -le $Total; $i++) {
        Add-Content "work.log" "change $i"
        if ($i -eq $BreakAt) { $script:Broken = $true }
        if ($script:Broken) { Set-Content "status.txt" "fail" -Encoding ascii }
        else { Set-Content "status.txt" "pass" -Encoding ascii }
        if ($i -eq $BreakAt) { Commit "commit ($i) break" } else { Commit "commit ($i)" }
    }
    Pop-Location
}

function Write-MergeRepo {
    param([string]$Name)
    Init-Repo $Name
    Write-PassTest
    for ($i = 1; $i -le 40; $i++) {
        Add-Content "main.log" "main $i"
        Set-Content "status.txt" "pass" -Encoding ascii
        Commit "main ($i)"
    }
    git checkout -q -b feature 2>$null
    for ($i = 1; $i -le 30; $i++) {
        Add-Content "feat.log" "feat $i"
        if ($i -eq 20) { $script:Broken = $true }
        if ($script:Broken) { Set-Content "status.txt" "fail" -Encoding ascii }
        else { Set-Content "status.txt" "pass" -Encoding ascii }
        if ($i -eq 20) { Commit "feature ($i) break" } else { Commit "feature ($i)" }
    }
    git checkout -q master 2>$null
    if (-not $?) { git checkout -q main 2>$null }
    git merge -q --no-edit feature -m "merge feature" 2>$null
    Pop-Location
}

function Write-LargeDiffRepo {
    param([string]$Name, [int]$Total, [int]$BreakAt, [int]$Files)
    Init-Repo $Name
    Write-PassTest
    New-Item -ItemType Directory -Force -Path "src" | Out-Null
    for ($f = 1; $f -le $Files; $f++) { Set-Content "src\f$f.txt" "v0" -Encoding ascii }
    Set-Content "status.txt" "pass" -Encoding ascii
    Commit "seed files"
    for ($i = 1; $i -le $Total; $i++) {
        for ($f = 1; $f -le $Files; $f++) { Add-Content "src\f$f.txt" "v$i" }
        if ($i -eq $BreakAt) { $script:Broken = $true }
        if ($script:Broken) { Set-Content "status.txt" "fail" -Encoding ascii }
        else { Set-Content "status.txt" "pass" -Encoding ascii }
        if ($i -eq $BreakAt) { Commit "update all ($i) break" } else { Commit "update all ($i)" }
    }
    Pop-Location
}

function Write-InteractionRepo {
    param([string]$Name)
    Init-Repo $Name
    @'
@echo off
echo x>>.execs
findstr /i "MODE=advanced" config.txt >nul 2>&1
if not %ERRORLEVEL%==0 exit /b 0
findstr /i "support_advanced" handler.txt >nul 2>&1
if %ERRORLEVEL%==0 (exit /b 0) else (exit /b 1)
'@ | Out-File "test.cmd" -Encoding ascii
    Set-Content "config.txt" "MODE=basic" -Encoding ascii
    Set-Content "handler.txt" "support_basic" -Encoding ascii
    Commit "initial config and handler"
    for ($i = 1; $i -le 5; $i++) {
        Add-Content "pad.log" "pad $i"
        Commit "padding ($i)"
    }
    Add-Content "handler.txt" "support_advanced"
    Commit "add advanced support to handler"
    for ($i = 1; $i -le 5; $i++) {
        Add-Content "pad.log" "pad2 $i"
        Commit "padding2 ($i)"
    }
    Set-Content "config.txt" "MODE=advanced" -Encoding ascii
    Commit "break enable advanced mode"
    for ($i = 1; $i -le 5; $i++) {
        Add-Content "pad.log" "pad3 $i"
        Commit "padding3 ($i)"
    }
    Pop-Location
}

function Write-DepChainRepo {
    param([string]$Name)
    Init-Repo $Name
    @'
@echo off
echo x>>.execs
set /p V=<vendor\lib\version.txt
findstr /C:"%V%" app\expect.txt >nul 2>&1
if %ERRORLEVEL%==0 (exit /b 0) else (exit /b 1)
'@ | Out-File "test.cmd" -Encoding ascii
    New-Item -ItemType Directory -Force -Path "vendor\lib","app" | Out-Null
    Set-Content "vendor\lib\version.txt" "v1" -Encoding ascii
    Set-Content "app\expect.txt" "requires v1" -Encoding ascii
    Commit "app expects vendor v1"
    for ($i = 1; $i -le 15; $i++) {
        Add-Content "pad.log" "pad $i"
        Commit "project work ($i)"
    }
    Set-Content "vendor\lib\version.txt" "v2-broken" -Encoding ascii
    Commit "break update vendored library to v2"
    for ($i = 1; $i -le 10; $i++) {
        Add-Content "pad.log" "after $i"
        Commit "post-update work ($i)"
    }
    Pop-Location
}

function Write-SlowRepo {
    param([string]$Name, [int]$Total, [int]$BreakAt)
    Init-Repo $Name
    @'
@echo off
echo x>>.execs
ping -n 1 -w 250 127.0.0.1 >nul 2>&1
findstr /i "fail" status.txt >nul 2>&1
if %ERRORLEVEL%==0 (exit /b 1) else (exit /b 0)
'@ | Out-File "test.cmd" -Encoding ascii
    for ($i = 1; $i -le $Total; $i++) {
        Add-Content "work.log" "slow change $i"
        if ($i -eq $BreakAt) { $script:Broken = $true }
        if ($script:Broken) { Set-Content "status.txt" "fail" -Encoding ascii }
        else { Set-Content "status.txt" "pass" -Encoding ascii }
        if ($i -eq $BreakAt) { Commit "slow commit ($i) break" } else { Commit "slow commit ($i)" }
    }
    Pop-Location
}

function Get-RootHash {
    return (git rev-list --max-parents=0 HEAD 2>$null | Select-Object -Last 1).Trim()
}

function Run-Bisect {
    param([string]$SampleName, [string]$Root = "")
    Remove-Item ".execs" -Force -ErrorAction SilentlyContinue
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    if (-not $Root) { $Root = Get-RootHash }
    git bisect start 2>$null
    git bisect bad 2>$null
    if ($Root) { git bisect good $Root 2>$null }
    $out = git bisect run cmd /c test.cmd 2>&1 | Out-String
    git bisect reset 2>$null | Out-Null
    $sw.Stop()
    $hash = ""
    foreach ($line in ($out -split "`n")) {
        if ($line -match "is the first bad commit") {
            if ($line -match "([0-9a-f]{7,40})") { $hash = $Matches[1]; break }
        }
    }
    $msg = ""
    if ($hash) { $msg = (git log -1 --format="%s" $hash 2>$null).Trim() }
    $execs = 0
    if (Test-Path ".execs") { $execs = @(Get-Content ".execs").Count }
    $out | Out-File "$ResultsDir\$SampleName-bisect.txt" -Encoding utf8
    return @{ Tool="git-bisect"; Ms=$sw.ElapsedMilliseconds; Execs=$execs; Hash=$hash; Msg=$msg }
}

function Run-Crux {
    param([string]$Mode, [string]$SampleName, [string]$Root = "")
    Remove-Item ".execs" -Force -ErrorAction SilentlyContinue
    if (-not $Root) { $Root = Get-RootHash }
    $range = "$Root..HEAD"
    $args2 = @("who","-c","test.cmd","-f",$range)
    if ($Mode -eq "fast") { $args2 += "--fast" }
    if ($Mode -eq "parallel") { $args2 += "--parallel" }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $out = (& $CruxPath @args2 2>&1) | Out-String
    $sw.Stop()
    $execs = 0
    if (Test-Path ".execs") { $execs = @(Get-Content ".execs").Count }
    $hash = ""
    $lines = $out -split "`n"
    foreach ($line in $lines) {
        if ($line -match "^(flip|commit):\s+([0-9a-f]+)\s+(.*)$") {
            $hash = $Matches[2]; break
        }
    }
    $msg = ""
    if ($hash -and $hash.Length -ge 7) { $msg = (git log -1 --format="%s" $hash 2>$null).Trim() }
    $name = "$SampleName-crux-$Mode"
    $out | Out-File "$ResultsDir\$name.txt" -Encoding utf8
    return @{ Tool="crux-$Mode"; Ms=$sw.ElapsedMilliseconds; Execs=$execs; Hash=$hash; Msg=$msg }
}

function Log {
    param($R, [string]$Scenario)
    $acc = "MISS"
    if ($R.Msg -match "break") { $acc = "CORRECT" }
    elseif ($R.Hash) { $acc = "WRONG" }
    "$Scenario|$($R.Tool)|$($R.Ms)|$($R.Execs)|$acc|$($R.Hash) $($R.Msg)" | Out-File "$ResultsDir\raw.tsv" -Append
    Write-Host ("  {0,-16} {1,8}ms  {2,4} runs  {3}" -f $R.Tool, $R.Ms, $R.Execs, $acc)
}

"scenario|tool|time_ms|test_runs|accuracy|found" | Out-File "$ResultsDir\raw.tsv"

Write-Host ""
Write-Host "=== CRUX BENCHMARK SUITE (fresh run) ===" -ForegroundColor Cyan
Write-Host "binary: $CruxPath ($([math]::Round((Get-Item $CruxPath).Length/1MB,2)) MB)"
Write-Host ""

foreach ($s in @(
    @{N="S1_linear_50"; F={Write-LinearRepo "S1_linear_50" 50 40}; Modes=@("normal","fast")},
    @{N="S2_linear_200"; F={Write-LinearRepo "S2_linear_200" 200 150}; Modes=@("normal","fast")},
    @{N="S3_linear_1000"; F={Write-LinearRepo "S3_linear_1000" 1000 750}; Modes=@("normal","fast")},
    @{N="S4_merge"; F={Write-MergeRepo "S4_merge"}; Modes=@("normal")},
    @{N="S5_large_diff"; F={Write-LargeDiffRepo "S5_large_diff" 50 30 100}; Modes=@("normal","fast")}
)) {
    Write-Host "--- $($s.N) ---" -ForegroundColor Yellow
    & $s.F
    Push-Location "$ReposDir\$($s.N)"
    Log (Run-Bisect $s.N) $s.N
    foreach ($m in $s.Modes) { Log (Run-Crux $m $s.N) $s.N }
    Pop-Location
}

Write-Host "--- S6_interaction ---" -ForegroundColor Yellow
Write-InteractionRepo "S6_interaction"
Push-Location "$ReposDir\S6_interaction"
Log (Run-Bisect "S6_interaction") "S6_interaction"
$r = Run-Crux "normal" "S6_interaction"
$suspects = (($r.Msg, (Get-Content "$ResultsDir\S6_interaction-crux-normal.txt" | Select-String "suspects")) -join " ")
"S6_interaction|crux-suspects|-|-|FEATURE|$suspects" | Out-File "$ResultsDir\raw.tsv" -Append
Log $r "S6_interaction"
Pop-Location

Write-Host "--- S7_dependency_chain ---" -ForegroundColor Yellow
Write-DepChainRepo "S7_dependency_chain"
Push-Location "$ReposDir\S7_dependency_chain"
Log (Run-Bisect "S7_dependency_chain") "S7_dependency_chain"
$r = Run-Crux "normal" "S7_dependency_chain"
$diffLines = (Get-Content "$ResultsDir\S7_dependency_chain-crux-normal.txt" | Where-Object { $_ -match "^\s+[+-]" } | Select-Object -First 4) -join " ; "
"S7_dependency_chain|crux-diff-preview|-|-|FEATURE|$diffLines" | Out-File "$ResultsDir\raw.tsv" -Append
Log $r "S7_dependency_chain"
Pop-Location

Write-Host "--- P1_slow_tests_parallel ---" -ForegroundColor Yellow
Write-SlowRepo "P1_slow_tests" 60 40
Push-Location "$ReposDir\P1_slow_tests"
Log (Run-Bisect "P1_slow_tests") "P1_slow_tests"
Log (Run-Crux "normal" "P1_slow_tests") "P1_slow_tests"
Log (Run-Crux "parallel" "P1_slow_tests") "P1_slow_tests"
Pop-Location

Write-Host "--- R1_real_repo_fd ---" -ForegroundColor Yellow
$fd = "$ReposDir\R1_fd_real"
if (-not (Test-Path $fd)) {
    git clone --quiet --depth 60 https://github.com/sharkdp/fd.git $fd 2>$null
}
if (Test-Path "$fd\.git") {
    Push-Location $fd
    $script:T = [datetime]::Parse("2026-01-01 12:00:00")
    git checkout -q -B bench HEAD~29 2>$null
    $script:Broken = $false
    $script:BaseHash = (git rev-parse HEAD).Trim()
    Set-Content ".gitignore" (Get-Content ".gitignore" -ErrorAction SilentlyContinue) -Encoding ascii
    Add-Content ".gitignore" "test.cmd`n.execs`n.crux/"
    Write-PassTest
    for ($i = 1; $i -le 25; $i++) {
        Add-Content "README.md" "bench note $i"
        if ($i -eq 18) { $script:Broken = $true }
        if ($script:Broken) { Set-Content "status.txt" "fail" -Encoding ascii }
        else { Set-Content "status.txt" "pass" -Encoding ascii }
        if ($i -eq 18) { Commit "synthetic bench commit ($i) break" } else { Commit "synthetic bench commit ($i)" }
    }
    Log (Run-Bisect "R1_fd_real" $script:BaseHash) "R1_fd_real"
    Log (Run-Crux "normal" "R1_fd_real" $script:BaseHash) "R1_fd_real"
    Log (Run-Crux "fast" "R1_fd_real" $script:BaseHash) "R1_fd_real"
    Pop-Location
} else {
    "R1_fd_real|clone-failed|-|-|SKIP|network unavailable" | Out-File "$ResultsDir\raw.tsv" -Append
    Write-Host "  clone failed, skipped" 
}

Write-Host ""
Write-Host "=== RAW RESULTS ===" -ForegroundColor Cyan
Get-Content "$ResultsDir\raw.tsv" | ForEach-Object { Write-Host "  $_" }

Clear-BenchEnv
