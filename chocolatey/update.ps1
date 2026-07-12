$ErrorActionPreference = 'Stop'

$packageName = "task-fighter"
# HTMLではなく、確実な情報を返すGitHubの公式APIを使用
$apiUrl = "https://api.github.com/repos/MoriokaReimen/Task-Fighter/releases/latest"
$currentDir  = $PSScriptRoot

Write-Host "Fetching latest release from GitHub API..." -ForegroundColor Cyan

# APIから最新リリースのJSONデータを取得
$response = Invoke-RestMethod -Uri $apiUrl -UseBasicParsing
$version = $response.tag_name.TrimStart('v')

# リリースアセット（zipファイル）の一覧からURLを抽出
$assets = $response.assets
$url32 = ($assets | Where-Object { $_.name -like "*i686*.zip" }).browser_download_url
$url64 = ($assets | Where-Object { $_.name -like "*x86_64*.zip" }).browser_download_url

if (-not $url32 -or -not $url64) {
    throw "Target zip files (i686 or x86_64) not found in the latest release assets."
}

Write-Host "Successfully Detected!" -ForegroundColor Green
Write-Host "Version: $version" -ForegroundColor Green
Write-Host "URL32:   $url32" -ForegroundColor Gray
Write-Host "URL64:   $url64" -ForegroundColor Gray

# chocolateyInstall.ps1 の書き換え
$installScriptPath = Join-Path $currentDir "tools\chocolateyInstall.ps1"
if (Test-Path $installScriptPath) {
    Write-Host "Updating chocolateyInstall.ps1..." -ForegroundColor Cyan
    $content = Get-Content $installScriptPath -Raw
    $content = $content -replace "(^[$]url32\s*=\s*)('.*')", "`$1'$url32'"
    $content = $content -replace "(^[$]url64\s*=\s*)('.*')", "`$1'$url64'"
    Set-Content -Path $installScriptPath -Value $content -Encoding UTF8
}

# .nuspec ファイルのバージョン書き換え
$nuspecPath = Join-Path $currentDir "$packageName.nuspec"
if (Test-Path $nuspecPath) {
    Write-Host "Updating $packageName.nuspec..." -ForegroundColor Cyan
    $content = Get-Content $nuspecPath -Raw
    $content = $content -replace "<version>.*</version>", "<version>$version</version>"
    Set-Content -Path $nuspecPath -Value $content -Encoding UTF8
}

Write-Host "Update completed successfully!" -ForegroundColor Green




