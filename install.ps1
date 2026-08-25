# crux installer for windows
# usage: powershell -ExecutionPolicy Bypass -File install.ps1
$ErrorActionPreference = "Stop"

$Repo = "Emran-goat/crux"
$Bindir = if ($env:CRUX_INSTALL_DIR) { $env:CRUX_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }

function Fetch($url, $out) {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $url -OutFile $out -UseBasicParsing
}

Write-Host "resolving latest release..."
$rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
$tag = $rel.tag_name
if (-not $tag) { throw "could not determine the latest release. is there one published?" }

$target = "x86_64-pc-windows-msvc"
$artifact = "crux-$target.zip"
$url = "https://github.com/$Repo/releases/download/$tag/$artifact"
Write-Host "downloading crux $tag ($target)"

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("crux-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
    Fetch $url (Join-Path $tmp $artifact)

    $sumUrl = "$url.sha256"
    try {
        Fetch $sumUrl (Join-Path $tmp "checksum")
        $expected = ((Get-Content (Join-Path $tmp "checksum") | Select-Object -First 1) -split '\s+')[0]
        $actual = (Get-FileHash (Join-Path $tmp $artifact) -Algorithm SHA256).Hash.ToLower()
        if ($expected.ToLower() -ne $actual) { throw "checksum mismatch for $artifact" }
        Write-Host "checksum ok"
    } catch {
        Write-Host "checksum file unavailable, skipping verification"
    }

    Expand-Archive -Path (Join-Path $tmp $artifact) -DestinationPath $tmp -Force

    New-Item -ItemType Directory -Force -Path $Bindir | Out-Null
    Move-Item -Force -Path (Join-Path $tmp "crux.exe") -Destination (Join-Path $Bindir "crux.exe")

    Write-Host ""
    Write-Host "installed: $(Join-Path $Bindir 'crux.exe') ($tag)"
    if (($env:PATH -split ";") -notcontains $Bindir) {
        Write-Host "note: $Bindir is not in your PATH. add it with:"
        Write-Host "      setx PATH `"$env:PATH;$Bindir`""
    }
    Write-Host "next: crux init"
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
