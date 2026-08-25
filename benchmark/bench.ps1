#!/usr/bin/env pwsh
$CruxPath = (Resolve-Path ".\target\release\crux.exe").Path
$ShExe = (Resolve-Path "C:\Program Files\Git\usr\bin\sh.exe").Path
$ShScript = (Resolve-Path ".\benchmark\bisect-test.sh").Path
$ShCmd = "`"$ShExe`" `"$ShScript`""
$Results = @()

function Bench($name, $tool, $ms, $iter, $acc, $extra) {
    $script:Results += [PSCustomObject]@{Scenario=$name; Tool=$tool; TimeMs=$ms; Iter=$iter; Acc=$acc; Extra=$extra}
    Write-Host ("  {0,-22} {1,8}ms  {2,4} iter  {3}  {4}" -f $tool, $ms, $iter, $acc, $extra)
}

function MakeRepo($name, $scriptblock) {
    $dir = "benchmark\repos\$name"
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Push-Location $dir
    $env:GIT_AUTHOR_NAME="bench"; $env:GIT_AUTHOR_EMAIL="bench@b.dev"
    $env:GIT_COMMITTER_NAME="bench"; $env:GIT_COMMITTER_EMAIL="bench@b.dev"
    git init -q 2>$null
    & $scriptblock | Out-Null
    Remove-Item Env:\GIT_AUTHOR_NAME -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_AUTHOR_EMAIL -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_NAME -ErrorAction SilentlyContinue
    Remove-Item Env:\GIT_COMMITTER_EMAIL -ErrorAction SilentlyContinue
    Pop-Location
    (Resolve-Path $dir).Path
}

function Run-Bisect($repoDir) {
    Push-Location $repoDir
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $root = (git rev-list --max-parents=0 HEAD 2>$null | Select-Object -First 1).Trim()
    git checkout -q master 2>$null
    git bisect start 2>$null; git bisect bad 2>$null; git bisect good $root 2>$null
    $out = git bisect run sh $ShScript 2>&1 | Out-String
    $found = ""
    foreach ($line in ($out -split "`n")) {
        if ($line -match "([0-9a-f]{6,40})\s+is the first bad commit") { $found = $Matches[1]; break }
    }
    $iters = ([regex]::Matches($out, "running '")).Count
    $msg = if ($found) { (git log -1 --format="%s" $found 2>$null).Trim() } else { "?" }
    git bisect reset 2>$null | Out-Null
    $sw.Stop()
    Pop-Location
    @{ Ms=$sw.ElapsedMilliseconds; Iters=$iters; Hash=$found; Msg=$msg }
}

function Run-Crux($repoDir, $cmd, $rangeArg) {
    Push-Location $repoDir
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $out = & $CruxPath who -c $cmd -f $rangeArg 2>&1
    $sw.Stop()
    $outStr = ($out -join "`n")
    $flip = ($outStr -split "`n" | Where-Object { $_ -match "^flip:" } | Select-Object -First 1)
    Pop-Location
    @{ Ms=$sw.ElapsedMilliseconds; Output=$flip; Full=$outStr }
}

Write-Host ""
Write-Host "=== CRUX BENCHMARK SUITE ===" -ForegroundColor Cyan
Write-Host "git bisect run vs crux who"
Write-Host ""

# S1: Small linear (50)
Write-Host "--- S1: Small linear (50, break at #40) ---" -ForegroundColor Yellow
$d = MakeRepo "s1" {
    Set-Content "status.txt" "pass"
    git add . -A; git commit -q -m "init"
    for ($i=1; $i -le 50; $i++) {
        if ($i -eq 40) { Set-Content "status.txt" "fail" }
        else { Add-Content "status.txt" " $i" }
        git add . -A; git commit -q -m "c$i"
    }
}
$b = Run-Bisect $d
$acc = if ($b.Hash -and $b.Msg -match "c40") {"CORRECT"} elseif ($b.Hash) {"HASH=$($b.Hash.Substring(0,7))"} else {"FAILED"}
Bench "small_50" "git-bisect" $b.Ms $b.Iters $acc "$($b.Hash.Substring(0,7)) $($b.Msg)"
$c = Run-Crux $d $ShCmd "HEAD~49..HEAD"
Bench "small_50" "crux" $c.Ms "?" "CHECK" $c.Output
Write-Host ""

# S2: Medium linear (200)
Write-Host "--- S2: Medium linear (200, break at #150) ---" -ForegroundColor Yellow
$d = MakeRepo "s2" {
    Set-Content "status.txt" "pass"
    git add . -A; git commit -q -m "init"
    for ($i=1; $i -le 200; $i++) {
        if ($i -eq 150) { Set-Content "status.txt" "fail" }
        else { Add-Content "status.txt" " $i" }
        git add . -A; git commit -q -m "c$i"
    }
}
$b = Run-Bisect $d
$acc = if ($b.Hash -and $b.Msg -match "c150") {"CORRECT"} elseif ($b.Hash) {"HASH=$($b.Hash.Substring(0,7))"} else {"FAILED"}
Bench "medium_200" "git-bisect" $b.Ms $b.Iters $acc "$($b.Hash.Substring(0,7)) $($b.Msg)"
$c = Run-Crux $d $ShCmd "HEAD~199..HEAD"
Bench "medium_200" "crux" $c.Ms "?" "CHECK" $c.Output
Write-Host ""

# S3: Large linear (1000)
Write-Host "--- S3: Large linear (1000, break at #750) ---" -ForegroundColor Yellow
$d = MakeRepo "s3" {
    Set-Content "status.txt" "pass"
    git add . -A; git commit -q -m "init"
    for ($i=1; $i -le 1000; $i++) {
        if ($i -eq 750) { Set-Content "status.txt" "fail" }
        else { Add-Content "status.txt" " $i" }
        git add . -A; git commit -q -m "c$i"
    }
}
$b = Run-Bisect $d
$acc = if ($b.Hash -and $b.Msg -match "c750") {"CORRECT"} elseif ($b.Hash) {"HASH=$($b.Hash.Substring(0,7))"} else {"FAILED"}
Bench "large_1000" "git-bisect" $b.Ms $b.Iters $acc "$($b.Hash.Substring(0,7)) $($b.Msg)"
$c = Run-Crux $d $ShCmd "HEAD~999..HEAD"
Bench "large_1000" "crux" $c.Ms "?" "CHECK" $c.Output
Write-Host ""

# S4: Merge history
Write-Host "--- S4: Merge (100, break in feature) ---" -ForegroundColor Yellow
$d = MakeRepo "s4" {
    Set-Content "status.txt" "pass"
    git add . -A; git commit -q -m "init"
    for ($i=1; $i -le 50; $i++) { git commit --allow-empty -q -m "main$i" }
    git checkout -q -b feature 2>$null
    for ($i=1; $i -le 50; $i++) {
        if ($i -eq 20) { Set-Content "status.txt" "fail"; git add . -A; git commit -q -m "break_feat$i" }
        else { git commit --allow-empty -q -m "feat$i" }
    }
    git checkout -q main 2>$null
    git merge -q --no-edit feature 2>$null
}
$b = Run-Bisect $d
$acc = if ($b.Hash -and $b.Msg -match "break_feat") {"CORRECT"} elseif ($b.Hash) {"HASH=$($b.Hash.Substring(0,7))"} else {"FAILED"}
Bench "merge_100" "git-bisect" $b.Ms $b.Iters $acc "$($b.Hash.Substring(0,7)) $($b.Msg)"
$c = Run-Crux $d $ShCmd "HEAD~99..HEAD"
Bench "merge_100" "crux" $c.Ms "?" "CHECK" $c.Output
Write-Host ""

# S5: Large diff
Write-Host "--- S5: Large diff (100 files, break at #30) ---" -ForegroundColor Yellow
$d = MakeRepo "s5" {
    Set-Content "status.txt" "pass"
    git add . -A; git commit -q -m "init"
    New-Item -ItemType Directory -Force -Path "src" | Out-Null
    for ($f=1; $f -le 100; $f++) { Set-Content "src\f$f.txt" "v0" }
    git add . -A; git commit -q -m "100files"
    for ($i=1; $i -le 50; $i++) {
        for ($f=1; $f -le 100; $f++) { Add-Content "src\f$f.txt" "c$i" }
        if ($i -eq 30) { Set-Content "status.txt" "fail" }
        git add . -A; git commit -q -m "upd$i"
    }
}
$b = Run-Bisect $d
$acc = if ($b.Hash -and $b.Msg -match "upd30") {"CORRECT"} elseif ($b.Hash) {"HASH=$($b.Hash.Substring(0,7))"} else {"FAILED"}
Bench "largediff" "git-bisect" $b.Ms $b.Iters $acc "$($b.Hash.Substring(0,7)) $($b.Msg)"
$c = Run-Crux $d $ShCmd "HEAD~49..HEAD"
Bench "largediff" "crux" $c.Ms "?" "CHECK" $c.Output
Write-Host ""

Write-Host "=== RESULTS ===" -ForegroundColor Cyan
$Results | Format-Table Scenario, Tool, TimeMs, Iter, Acc, Extra -AutoSize
