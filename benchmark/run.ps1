#Requires -Version 5.1
param(
    [string]$CruxPath = ".\target\release\crux.exe"
)

$ErrorActionPreference = "SilentlyContinue"
$ResultsDir = "benchmark\results"
$ReposDir = "benchmark\repos"

if (Test-Path $ResultsDir) { Remove-Item "$ResultsDir\*" -Force }
else { New-Item -ItemType Directory -Force -Path $ResultsDir | Out-Null }
if (-not (Test-Path $ReposDir)) { New-Item -ItemType Directory -Force -Path $ReposDir | Out-Null }

$CruxPath = (Resolve-Path $CruxPath).Path

function Log-Result {
    param([string]$Scenario, [string]$Tool, [int]$TimeMs, [string]$Iterations, [string]$Accuracy, [string]$Extra)
    "$Scenario|$Tool|$TimeMs|$Iterations|$Accuracy|$Extra" | Out-File "$ResultsDir\raw.tsv" -Append
    Write-Host ("  {0,-22} {1,8}ms  {2,4} iter  {3}  {4}" -f $Tool, $TimeMs, $Iterations, $Accuracy, $Extra)
}

function Set-BenchEnv {
    $env:GIT_AUTHOR_NAME="bench"; $env:GIT_AUTHOR_EMAIL="bench@b.dev"
    $env:GIT_COMMITTER_NAME="bench"; $env:GIT_COMMITTER_EMAIL="bench@b.dev"
}

function Clear-BenchEnv {
    Remove-Item Env:\GIT_AUTHOR_NAME -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_AUTHOR_EMAIL -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_NAME -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_EMAIL -ErrorAction SilentlyContinue
}

function Write-BisectScript {
    param([string]$Dir)
    @'
@echo off
git log -1 --format=%s HEAD | findstr /i "break" >nul
if %ERRORLEVEL%==0 (exit 1) else (exit 0)
'@ | Out-File "$Dir\test.cmd" -Encoding ascii -NoNewline
}

function Create-LinearRepo {
    param([string]$Name, [int]$Total, [int]$BreakAt)
    $dir = "$ReposDir\$Name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    Set-BenchEnv
    git init -q 2>$null
    git commit --allow-empty -q -m "initial" 2>$null
    for ($i = 1; $i -le $Total; $i++) {
        if ($i -eq $BreakAt) {
            git commit --allow-empty -q -m "break behavior ($i)" 2>$null
        } else {
            git commit --allow-empty -q -m "normal commit ($i)" 2>$null
        }
    }
    Write-BisectScript $dir
    Clear-BenchEnv
    Pop-Location
}

function Create-MergeRepo {
    param([string]$Name, [int]$Total, [int]$BreakAt)
    $dir = "$ReposDir\$Name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    Set-BenchEnv
    git init -q 2>$null
    git commit --allow-empty -q -m "initial" 2>$null
    $mainCount = [math]::Floor($Total / 2)
    for ($i = 1; $i -le $mainCount; $i++) {
        git commit --allow-empty -q -m "main commit ($i)" 2>$null
    }
    git checkout -q -b feature 2>$null
    $featCount = $Total - $mainCount
    for ($i = 1; $i -le $featCount; $i++) {
        if ($i -eq ($BreakAt - $mainCount)) {
            git commit --allow-empty -q -m "break in feature ($i)" 2>$null
        } else {
            git commit --allow-empty -q -m "feature commit ($i)" 2>$null
        }
    }
    git checkout -q main 2>$null
    git merge -q --no-edit feature 2>$null
    Write-BisectScript $dir
    Clear-BenchEnv
    Pop-Location
}

function Create-InteractionRepo {
    param([string]$Name)
    $dir = "$ReposDir\$Name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    Set-BenchEnv
    git init -q 2>$null
    git commit --allow-empty -q -m "initial" 2>$null
    git commit --allow-empty -q -m "commit A (harmless alone)" 2>$null
    git commit --allow-empty -q -m "commit B (harmless alone)" 2>$null
    git commit --allow-empty -q -m "commit C (interaction trigger)" 2>$null
    for ($i = 1; $i -le 10; $i++) {
        git commit --allow-empty -q -m "padding commit ($i)" 2>$null
    }
    Write-BisectScript $dir
    Clear-BenchEnv
    Pop-Location
}

function Create-DepChainRepo {
    param([string]$Name)
    $dir = "$ReposDir\$Name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    Set-BenchEnv
    git init -q 2>$null
    git commit --allow-empty -q -m "initial" 2>$null
    New-Item -ItemType Directory -Force -Path "vendor\lib" | Out-Null
    Set-Content "vendor\lib\version.txt" "v1"
    git add . -A 2>$null
    git commit -q -m "add vendor v1" 2>$null
    for ($i = 1; $i -le 20; $i++) {
        git commit --allow-empty -q -m "project commit ($i)" 2>$null
    }
    Set-Content "vendor\lib\version.txt" "v2-broken"
    git add . -A 2>$null
    git commit -q -m "update vendor v2 (breaks)" 2>$null
    for ($i = 1; $i -le 10; $i++) {
        git commit --allow-empty -q -m "project commit after ($i)" 2>$null
    }
    Write-BisectScript $dir
    Clear-BenchEnv
    Pop-Location
}

function Create-LargeDiffRepo {
    param([string]$Name, [int]$Total, [int]$BreakAt, [int]$Files)
    $dir = "$ReposDir\$Name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    Set-BenchEnv
    git init -q 2>$null
    git commit --allow-empty -q -m "initial" 2>$null
    New-Item -ItemType Directory -Force -Path "src" | Out-Null
    for ($f = 1; $f -le $Files; $f++) {
        Set-Content "src\file_$f.txt" "line1"
    }
    git add . -A 2>$null
    git commit -q -m "create $Files files" 2>$null
    for ($i = 1; $i -le $Total; $i++) {
        for ($f = 1; $f -le $Files; $f++) {
            Add-Content "src\file_$f.txt" "change $i $f"
        }
        if ($i -eq $BreakAt) {
            Add-Content "src\file_1.txt" "BREAK"
        }
        git add . -A 2>$null
        git commit -q -m "update all files ($i)" 2>$null
    }
    Write-BisectScript $dir
    Clear-BenchEnv
    Pop-Location
}

function Time-Bisect {
    param([string]$RepoDir, [string]$TestScript)
    Push-Location $RepoDir
    $sw = [System.Diagnostics.Stopwatch]::StartNew()

    $rootHash = (git rev-list --max-parents=0 HEAD 2>$null | Select-Object -First 1).Trim()
    git bisect start 2>$null
    git bisect bad 2>$null
    if ($rootHash) { git bisect good $rootHash 2>$null }

    $bisectOutput = git bisect run $TestScript 2>&1 | Out-String

    $foundHash = ""
    $foundMsg = ""
    $lines = $bisectOutput -split "`n"
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match "is the first bad commit") {
            if ($lines[$i] -match "^([0-9a-f]{6,40})") {
                $foundHash = $Matches[1]
            } elseif ($i -gt 0 -and $lines[$i-1] -match "([0-9a-f]{6,40})") {
                $foundHash = $Matches[1]
            }
            break
        }
    }

    if ($foundHash) {
        $foundMsg = (git log -1 --format="%s" $foundHash 2>$null).Trim()
    }

    $iterCount = ([regex]::Matches($bisectOutput, "running '")).Count

    git bisect reset 2>$null | Out-Null
    $sw.Stop()
    Pop-Location
    return @{ TimeMs = $sw.ElapsedMilliseconds; Iterations = $iterCount; Hash = $foundHash; Msg = $foundMsg }
}

function Time-Crux {
    param([string]$RepoDir, [string]$Cmd, [string]$ExtraArgs)
    Push-Location $RepoDir
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & $CruxPath who $Cmd ($ExtraArgs -split ' ') 2>&1
    $sw.Stop()
    Pop-Location
    return @{ TimeMs = $sw.ElapsedMilliseconds; Output = ($output -join "`n") }
}

Write-Host ""
Write-Host "=== CRUX BENCHMARK SUITE ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Comparing: git bisect run vs crux who"
Write-Host "Binary: $CruxPath"
Write-Host ""

"scenario|tool|time_ms|iterations|accuracy|extra" | Out-File "$ResultsDir\raw.tsv"

# --- Scenario 1: Small linear ---
Write-Host "--- Scenario 1: Small linear (50 commits, break at #40) ---" -ForegroundColor Yellow
Create-LinearRepo "small_linear" 50 40
$repo = (Resolve-Path "$ReposDir\small_linear").Path
$testScript = (Resolve-Path "$repo\test.cmd").Path
Write-Host ("  {0,-22} {1,8}  {2,4}  {3}" -f "TOOL", "TIME", "ITER", "ACCURACY")
$b = Time-Bisect $repo $testScript
$acc = if ($b.Msg -match "break") { "CORRECT" } else { "WRONG" }
Log-Result "small_linear_50" "git-bisect" $b.TimeMs $b.Iterations $acc "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "echo PASS" "--from HEAD~49..HEAD"
$flip = ($c.Output -split "`n" | Where-Object { $_ -match "^flip:" } | Select-Object -First 1)
$flipHash = if ($flip -match "flip:\s+(\S+)") { $Matches[1] } else { "" }
$flipMsg = if ($flipHash) { (git -C $repo log -1 --format="%s" $flipHash 2>$null).Trim() } else { "" }
$acc2 = if ($flipMsg -match "break") { "CORRECT" } else { "CHECK" }
Log-Result "small_linear_50" "crux" $c.TimeMs "?" $acc2 "$flipHash $flipMsg"
Write-Host ""

# --- Scenario 2: Medium linear ---
Write-Host "--- Scenario 2: Medium linear (200 commits, break at #150) ---" -ForegroundColor Yellow
Create-LinearRepo "medium_linear" 200 150
$repo = (Resolve-Path "$ReposDir\medium_linear").Path
$testScript = (Resolve-Path "$repo\test.cmd").Path
$b = Time-Bisect $repo $testScript
$acc = if ($b.Msg -match "break") { "CORRECT" } else { "WRONG" }
Log-Result "medium_linear_200" "git-bisect" $b.TimeMs $b.Iterations $acc "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "echo PASS" "--from HEAD~199..HEAD"
$flip = ($c.Output -split "`n" | Where-Object { $_ -match "^flip:" } | Select-Object -First 1)
$flipHash = if ($flip -match "flip:\s+(\S+)") { $Matches[1] } else { "" }
$flipMsg = if ($flipHash) { (git -C $repo log -1 --format="%s" $flipHash 2>$null).Trim() } else { "" }
$acc2 = if ($flipMsg -match "break") { "CORRECT" } else { "CHECK" }
Log-Result "medium_linear_200" "crux" $c.TimeMs "?" $acc2 "$flipHash $flipMsg"
Write-Host ""

# --- Scenario 3: Large linear ---
Write-Host "--- Scenario 3: Large linear (1000 commits, break at #750) ---" -ForegroundColor Yellow
Create-LinearRepo "large_linear" 1000 750
$repo = (Resolve-Path "$ReposDir\large_linear").Path
$testScript = (Resolve-Path "$repo\test.cmd").Path
$b = Time-Bisect $repo $testScript
$acc = if ($b.Msg -match "break") { "CORRECT" } else { "WRONG" }
Log-Result "large_linear_1000" "git-bisect" $b.TimeMs $b.Iterations $acc "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "echo PASS" "--from HEAD~999..HEAD"
$flip = ($c.Output -split "`n" | Where-Object { $_ -match "^flip:" } | Select-Object -First 1)
$flipHash = if ($flip -match "flip:\s+(\S+)") { $Matches[1] } else { "" }
$flipMsg = if ($flipHash) { (git -C $repo log -1 --format="%s" $flipHash 2>$null).Trim() } else { "" }
$acc2 = if ($flipMsg -match "break") { "CORRECT" } else { "CHECK" }
Log-Result "large_linear_1000" "crux" $c.TimeMs "?" $acc2 "$flipHash $flipMsg"
Write-Host ""

# --- Scenario 4: Merge history ---
Write-Host "--- Scenario 4: Merge history (100 commits, break in feature) ---" -ForegroundColor Yellow
Create-MergeRepo "merge_history" 100 70
$repo = (Resolve-Path "$ReposDir\merge_history").Path
$testScript = (Resolve-Path "$repo\test.cmd").Path
$b = Time-Bisect $repo $testScript
$acc = if ($b.Msg -match "break") { "CORRECT" } else { "WRONG" }
Log-Result "merge_history_100" "git-bisect" $b.TimeMs $b.Iterations $acc "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "echo PASS" "--from HEAD~99..HEAD"
$flip = ($c.Output -split "`n" | Where-Object { $_ -match "^flip:" } | Select-Object -First 1)
$flipHash = if ($flip -match "flip:\s+(\S+)") { $Matches[1] } else { "" }
$flipMsg = if ($flipHash) { (git -C $repo log -1 --format="%s" $flipHash 2>$null).Trim() } else { "" }
$acc2 = if ($flipMsg -match "break") { "CORRECT" } else { "CHECK" }
Log-Result "merge_history_100" "crux" $c.TimeMs "?" $acc2 "$flipHash $flipMsg"
Write-Host ""

# --- Scenario 5: Large diff ---
Write-Host "--- Scenario 5: Large diff (100 files/commit, break at #30) ---" -ForegroundColor Yellow
Create-LargeDiffRepo "large_diff" 50 30 100
$repo = (Resolve-Path "$ReposDir\large_diff").Path
$testScript = (Resolve-Path "$repo\test.cmd").Path
$b = Time-Bisect $repo $testScript
$acc = if ($b.Msg -match "break") { "CORRECT" } else { "WRONG" }
Log-Result "large_diff_50" "git-bisect" $b.TimeMs $b.Iterations $acc "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "echo PASS" "--from HEAD~49..HEAD"
$flip = ($c.Output -split "`n" | Where-Object { $_ -match "^flip:" } | Select-Object -First 1)
$flipHash = if ($flip -match "flip:\s+(\S+)") { $Matches[1] } else { "" }
$flipMsg = if ($flipHash) { (git -C $repo log -1 --format="%s" $flipHash 2>$null).Trim() } else { "" }
$acc2 = if ($flipMsg -match "break") { "CORRECT" } else { "CHECK" }
Log-Result "large_diff_50" "crux" $c.TimeMs "?" $acc2 "$flipHash $flipMsg"
Write-Host ""

# --- Scenario 6: Interaction fault ---
Write-Host "--- Scenario 6: Interaction fault (crux-exclusive) ---" -ForegroundColor Yellow
Create-InteractionRepo "interaction_fault"
$repo = (Resolve-Path "$ReposDir\interaction_fault").Path
$b = Time-Bisect $repo "$repo\test.cmd"
Log-Result "interaction_fault" "git-bisect" $b.TimeMs $b.Iterations "N/A (single-flip only)" "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "echo PASS" "--from HEAD~13..HEAD"
$inter = ($c.Output -split "`n" | Where-Object { $_ -match "interaction:" } | Select-Object -First 1)
Log-Result "interaction_fault" "crux-who" $c.TimeMs "?" "FEATURE" $inter
Write-Host ""

# --- Scenario 7: Dependency chain ---
Write-Host "--- Scenario 7: Dependency chain (crux-exclusive) ---" -ForegroundColor Yellow
Create-DepChainRepo "dep_chain"
$repo = (Resolve-Path "$ReposDir\dep_chain").Path
$b = Time-Bisect $repo "$repo\test.cmd"
$acc = if ($b.Msg -match "vendor|break") { "CORRECT" } else { "CHECK" }
Log-Result "dep_chain" "git-bisect" $b.TimeMs $b.Iterations $acc "$($b.Hash) $($b.Msg)"
$c = Time-Crux $repo "grep -r 'v2-broken' vendor/" "--from HEAD~31..HEAD"
$upstream = ($c.Output -split "`n" | Where-Object { $_ -match "upstream:" } | Select-Object -First 1)
Log-Result "dep_chain" "crux-who" $c.TimeMs "?" "FEATURE" $upstream
Write-Host ""

# --- Summary ---
Write-Host "=== RESULTS SUMMARY ===" -ForegroundColor Cyan
Write-Host ""
$raw = Get-Content "$ResultsDir\raw.tsv"
foreach ($line in $raw) {
    $parts = $line -split '\|'
    Write-Host ("  {0,-24} {1,-14} {2,8}ms  {3,4} iter  {4}  {5}" -f $parts[0], $parts[1], $parts[2], $parts[3], $parts[4], $parts[5])
}
Write-Host ""
Write-Host "Full results: $ResultsDir\raw.tsv"
