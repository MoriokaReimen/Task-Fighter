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
$asset32 = $assets | Where-Object { $_.name -like "*i686*.zip" }
$asset64 = $assets | Where-Object { $_.name -like "*x86_64*.zip" }

$url32 = $asset32.browser_download_url
$url64 = $asset64.browser_download_url

if (-not $url32 -or -not $url64) {
    throw "Target zip files (i686 or x86_64) not found in the latest release assets."
}

# --- ここからチェックサムのAPI抽出処理を追加 ---
Write-Host "Fetching checksums from API assets..." -ForegroundColor Cyan

# アセット名に "checksum" や "sha256" が含まれるテキストファイルを検索
$checksumAsset = $assets | Where-Object { $_.name -like "*checksum*" -or $_.name -like "*sha256*" -and $_.name -like "*.txt" }

$newChecksum32 = $null
$newChecksum64 = $null

if ($checksumAsset) {
    # チェックサムファイルの中身（テキスト）を直接取得
    $checksumTxtUrl = $checksumAsset.browser_download_url
    $checksumContent = Invoke-RestMethod -Uri $checksumTxtUrl -UseBasicParsing
    
    # テキスト内からそれぞれのzipファイル名に対応するハッシュ値を正規表現で抽出
    # 一般的な「ハッシュ値  ファイル名」の形式に対応
    if ($checksumContent -match "([a-fA-F0-9]{64})\s+.*$($asset32.name)") {
        $newChecksum32 = $Matches[1].ToUpper()
    }
    if ($checksumContent -match "([a-fA-F0-9]{64})\s+.*$($asset64.name)") {
        $newChecksum64 = $Matches[1].ToUpper()
    }
}

# 万が一APIから取得できなかった場合のフォールバック（手動計算）
if (-not $newChecksum32 -or -not $newChecksum64) {
    Write-Warning "Could not extract checksums from API. Downloading files to calculate..."
    $tempZip32 = Join-Path $env:TEMP "task-fighter-32.tmp"
    $tempZip64 = Join-Path $env:TEMP "task-fighter-64.tmp"
    
    Invoke-WebRequest -Uri $url32 -OutFile $tempZip32 -UseBasicParsing
    Invoke-WebRequest -Uri $url64 -OutFile $tempZip64 -UseBasicParsing
    
    if (-not $newChecksum32) { $newChecksum32 = (Get-FileHash -Path $tempZip32 -Algorithm SHA256).Hash.ToUpper() }
    if (-not $newChecksum64) { $newChecksum64 = (Get-FileHash -Path $tempZip64 -Algorithm SHA256).Hash.ToUpper() }
    
    Remove-Item $tempZip32, $tempZip64 -ErrorAction SilentlyContinue
}
# --- ここまで抽出処理 ---

Write-Host "Successfully Detected!" -ForegroundColor Green
Write-Host "Version:    $version" -ForegroundColor Green
Write-Host "URL32:      $url32" -ForegroundColor Gray
Write-Host "Checksum32: $newChecksum32" -ForegroundColor Gray
Write-Host "URL64:      $url64" -ForegroundColor Gray
Write-Host "Checksum64: $newChecksum64" -ForegroundColor Gray

# chocolateyInstall.ps1 の書き換え
$installScriptPath = Join-Path $currentDir "tools\chocolateyInstall.ps1"
if (Test-Path $installScriptPath) {
    Write-Host "Updating chocolateyInstall.ps1..." -ForegroundColor Cyan
    $content = Get-Content $installScriptPath -Raw
    
    # 各変数定義の行頭のみを狙い撃ちして置換
    $content = $content -replace '(?m)(^\s*\$url32\s*=\s*)(''.*?'')', "`$1'$url32'"
    $content = $content -replace '(?m)(^\s*\$url64\s*=\s*)(''.*?'')', "`$1'$url64'"
    $content = $content -replace '(?m)(^\s*\$checksum32\s*=\s*)(''.*?'')', "`$1'$newChecksum32'"
    $content = $content -replace '(?m)(^\s*\$checksum64\s*=\s*)(''.*?'')', "`$1'$newChecksum64'"
    
    Set-Content -Path $installScriptPath -Value $content -Encoding UTF8
}

# VERIFICATION.txt の書き換え
$verificationPath = Join-Path $currentDir "tools\VERIFICATION.txt"
if (Test-Path $verificationPath) {
    Write-Host "Updating VERIFICATION.txt..." -ForegroundColor Cyan
    $vContent = Get-Content $verificationPath -Raw

    # 1. バージョン表記の更新 (例: Version 8.2.4 や version 8.2.4 にマッチ)
    $vContent = $vContent -replace "(?i)(version\s+)[0-9.]+", "`${1}$version"

    # 2. i686 (32-bit) のハッシュ値の更新
    # 行頭から「- i686」で始まる行のハッシュ値部分（コロンの直後）を書き換え
    $vContent = $vContent -replace "(?m)(^\s*-\s*i686.*:\s*)[A-Fa-f0-9]{64}", "`${1}$newChecksum32"

    # 3. x86_64 (64-bit) のハッシュ値の更新
    # 行頭から「- x86_64」で始まる行のハッシュ値部分（コロンの直後）を書き換え
    $vContent = $vContent -replace "(?m)(^\s*-\s*x86_64.*:\s*)[A-Fa-f0-9]{64}", "`${1}$newChecksum64"

    Set-Content -Path $verificationPath -Value $vContent -Encoding UTF8
}

# .nuspec ファイルのバージョンとリリースノートの書き換え
$nuspecPath = Join-Path $currentDir "$packageName.nuspec"
if (Test-Path $nuspecPath) {
    Write-Host "Updating $packageName.nuspec..." -ForegroundColor Cyan
    $content = Get-Content $nuspecPath -Raw
    
    # 1. <version>タグの書き換え
    $content = $content -replace "<version>.*?</version>", "<version>$version</version>"
    
    # 2. <releaseNotes>タグの書き換え（もしnuspecに定義されていれば自動更新）
    $content = $content -replace "<releaseNotes>.*?</releaseNotes>", "<releaseNotes>https://github.com/MoriokaReimen/Task-Fighter/releases/tag/v$version</releaseNotes>"
    
    Set-Content -Path $nuspecPath -Value $content -Encoding UTF8
}


