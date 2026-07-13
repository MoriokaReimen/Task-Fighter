$ErrorActionPreference = 'Stop'

$packageName = 'task-fighter'
$toolsDir    = $PSScriptRoot
$url32       = 'https://github.com/MoriokaReimen/Task-Fighter/releases/download/8.1.0/task-fighter-8.1.0-i686-pc-windows-msvc.zip'
$url64       = 'https://github.com/MoriokaReimen/Task-Fighter/releases/download/8.1.0/task-fighter-8.1.0-x86_64-pc-windows-msvc.zip'
$checksum32  = '2839A24C3D277877D1F2A2238B7DEA63EEE3317C022885CE42F9B0DF370B2740'
$checksum64  = '3DF7BCE7AA5B6AF2576B9645939D694F2CAD1E3AE24A4A83E698ECC8CA41F021'

$packageArgs = @{
  packageName    = $packageName
  url            = $url32
  url64Bit       = $url64
  checksum       = $checksum32
  checksum64     = $checksum64
  checksumType   = 'sha256'
  checksumType64 = 'sha256'
  unzipLocation  = Split-Path $MyInvocation.MyCommand.Definition
}

Install-ChocolateyZipPackage @packageArgs

$exeFile = Get-ChildItem -Path $toolsDir -Filter "task-fighter.exe" -Recurse | Select-Object -First 1

if (-not $exeFile) {
    throw "task-fighter.exe is not found."
}

$exePath = $exeFile.FullName

$desktopPath = [Environment]::GetFolderPath('Desktop')
$shortcutFile = Join-Path $desktopPath "Task-Fighter.lnk"

Write-Host "Creating desktop shortcut: $shortcutFile" -ForegroundColor Cyan
Install-ChocolateyShortcut -ShortcutFilePath $shortcutFile -TargetPath $exePath
