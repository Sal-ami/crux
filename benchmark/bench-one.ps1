param(
    [string]$Repo,
    [string]$Modes = "bisect,normal,fast",
    [string]$CruxPath = "C:\Users\fetit\AppData\Local\Temp\opencode\crux-bench\crux.exe",
    [string]$Work = "C:\Users\fetit\AppData\Local\Temp\opencode\crux-bench"
)
$ErrorActionPreference = "SilentlyContinue"
$ResultsDir = "$Work\results"
Set-Location "$Work\repos\$Repo"

function Test-Script {
    Remove-Item ".execs" -Force -ErrorAction SilentlyContinue
}
function Count-Execs {
    if (Test-Path ".execs") { return @(Get-Content ".execs").Count }
    return 0
}

if ($Modes -match "bisect") {
# git-bisect
Test-Script
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$root = (git rev-list --max-parents=0 HEAD 2>$null | Select-Object -Last 1).Trim()
git bisect start 2>$null
git bisect bad 2>$null
if ($root) { git bisect good $root 2>$null }
$out = git bisect run sh bisect-test.sh 2>&1 | Out-String
git bisect reset 2>$null | Out-Null
$sw.Stop()
$bHash = ""
foreach ($line in ($out -split "`n")) {
    if ($line -match "is the first bad commit") {
        if ($line -match "^([0-9a-f]{7,40})") { $bHash = $Matches[1]; break }
    }
}
$bMsg = ""
if ($bHash) { $bMsg = (git log -1 --format="%s" $bHash 2>$null).Trim() }
$out | Out-File "$ResultsDir\$Repo-bisect.txt" -Encoding utf8
"$Repo|git-bisect|$($sw.ElapsedMilliseconds)|$(Count-Execs)|$bMsg" | Out-File "$ResultsDir\raw.tsv" -Append
Write-Host ("  {0,-16} {1,8}ms {2,4} runs  {3}" -f "git-bisect", $sw.ElapsedMilliseconds, (Count-Execs), $bMsg)
}

foreach ($mode in @("normal","fast","parallel")) {
    if ($Modes -notmatch $mode) { continue }
    Test-Script
    $root = (git rev-list --max-parents=0 HEAD 2>$null | Select-Object -Last 1).Trim()
    $range = "$root..HEAD"
    $a = @("who","-c","test.cmd","-f",$range)
    if ($mode -eq "fast") { $a += "--fast" }
    if ($mode -eq "parallel") { $a += "--parallel" }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $out = (& $CruxPath @a 2>&1) | Out-String
    $sw.Stop()
    $hash = ""
    foreach ($line in ($out -split "`n")) {
        if ($line -match "^(flip|commit):\s+([0-9a-f]+)\s+(.*)$") { $hash = $Matches[2]; break }
    }
    $msg = ""
    if ($hash -and $hash.Length -ge 7) { $msg = (git log -1 --format="%s" $hash 2>$null).Trim() }
    $out | Out-File "$ResultsDir\$Repo-crux-$mode.txt" -Encoding utf8
    "$Repo|crux-$mode|$($sw.ElapsedMilliseconds)|$(Count-Execs)|$msg" | Out-File "$ResultsDir\raw.tsv" -Append
    Write-Host ("  {0,-16} {1,8}ms {2,4} runs  {3}" -f "crux-$mode", $sw.ElapsedMilliseconds, (Count-Execs), $msg)
}
