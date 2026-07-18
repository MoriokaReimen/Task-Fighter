$ErrorActionPreference = 'Stop'

$packageName = 'task-fighter'
$toolsDir    = $PSScriptRoot
$url32       = 'https://github.com/MoriokaReimen/Task-Fighter/releases/download/8.3.0/task-fighter-8.3.0-i686-pc-windows-msvc.zip'
$url64       = 'https://github.com/MoriokaReimen/Task-Fighter/releases/download/8.3.0/task-fighter-8.3.0-x86_64-pc-windows-msvc.zip'
$checksum32  = 'F620F6269E40146F3C2F9BADEC7985C4C88D9AA8E0AE903DCEADD822BB8F3FA2'
$checksum64  = '9EAE318C30974864F80ED8F4CA65CC0689597E0A3430623A27B7C8E64848BD5A'

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
