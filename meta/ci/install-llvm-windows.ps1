param(
	[string]$Version = "20.1.8",
	[string]$InstallRoot = "C:\LLVM-20"
)

set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Variables
$Uri = "https://github.com/llvm/llvm-project/releases/download/llvmorg-$Version/clang+llvm-$Version-x86_64-pc-windows-msvc.tar.xz"
$Temp = [IO.Path]::GetFullPath($env:TEMP)
$Download = Join-Path $Temp "clang+llvm-$Version-x86_64-pc-windows-msvc.tar.xz"
$ExtractRoot = Join-Path $Temp "llvm-extract-$Version"

Write-Host "Downloading LLVM $Version from $Uri" -ForegroundColor Cyan
Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Download

if (Test-Path $ExtractRoot) { Remove-Item -Recurse -Force $ExtractRoot }
New-Item -ItemType Directory -Path $ExtractRoot | Out-Null

# Extract archive
Write-Host "Extracting archive -> folder" -ForegroundColor Cyan
$sevenZip = (Get-Command 7z).Path
# Stream: xz -> stdout, then untar from stdin into $ExtractRoot
& $sevenZip x -so "$Download" | & $sevenZip x -si -ttar "-o$ExtractRoot" | Out-Null

# Inside the tar there is a top-level directory named like clang+llvm-<ver>-x86_64-pc-windows-msvc
$InnerDir = Get-ChildItem -Path $ExtractRoot | Where-Object { $_.PSIsContainer } | Select-Object -First 1
if (-not $InnerDir) { throw "Failed to find extracted LLVM directory in $ExtractRoot" }

Write-Host "Installing to $InstallRoot" -ForegroundColor Cyan
if (Test-Path $InstallRoot) { Remove-Item -Recurse -Force $InstallRoot }
Move-Item -Force $InnerDir.FullName $InstallRoot

# Set env var
if ($env:GITHUB_ENV) { Add-Content -Path $env:GITHUB_ENV -Value "LLVM_SYS_201_PREFIX=$InstallRoot" }

# Add bin to PATH for current process and GHA
$BinPath = Join-Path $InstallRoot 'bin'
$env:PATH = "$BinPath;$env:PATH"
if ($env:GITHUB_PATH) { Add-Content -Path $env:GITHUB_PATH -Value $BinPath }

# Cleanup
Write-Host "Cleaning up temporary files" -ForegroundColor Cyan
Remove-Item -Force $Download -ErrorAction SilentlyContinue
if (Test-Path $ExtractRoot) { Remove-Item -Recurse -Force $ExtractRoot }

Write-Host "LLVM $Version installed at $InstallRoot" -ForegroundColor Green
