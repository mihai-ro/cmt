# Install the latest cmt release binary on Windows.
$ErrorActionPreference = "Stop"
$repo = "mihai-ro/cmt"
$binDir = if ($env:CMT_BIN_DIR) { $env:CMT_BIN_DIR } else { "$env:LOCALAPPDATA\Programs\cmt" }

$arch = if ([Environment]::Is64BitOperatingSystem) {
  if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
} else { throw "Unsupported architecture" }

$target = "$arch-pc-windows-msvc"
$url = "https://github.com/$repo/releases/latest/download/cmt-$target.exe"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
$dest = Join-Path $binDir "cmt.exe"
Write-Host "Downloading cmt ($target)..."
Invoke-WebRequest -Uri $url -OutFile $dest
Write-Host "Installed to $dest"
Write-Host "Add $binDir to your PATH if it is not already."
