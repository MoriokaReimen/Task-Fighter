$ErrorActionPreference = 'Stop'

$packageName = 'task-fighter'
$toolsDir    = $PSScriptRoot
$url32       = 'https://github.com/MoriokaReimen/Task-Fighter/releases/download/8.3.0/task-fighter-8.3.0-i686-pc-windows-msvc.zip'
$url64       = 'https://github.com/MoriokaReimen/Task-Fighter/releases/download/8.3.0/task-fighter-8.3.0-x86_64-pc-windows-msvc.zip'
$checksum32  = '47939D9D0C3CB9CFE2514916EBD11C8796787EF489D35CA9BA1C5C2147403A71'
$checksum64  = '71B65395735D4687009CA9DC75D1D255A526FC7944C085CCFD72F014CEBD113F'

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
