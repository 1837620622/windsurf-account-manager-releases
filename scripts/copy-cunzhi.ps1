# 复制 cunzhi exe 文件到 src-tauri/cunzhi 目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir
$cunzhiDir = Join-Path $projectRoot "src-tauri\cunzhi"

# 源文件路径
$mcpExe = "F:\Trace\TEST\windsurf\Windsurf Infinite Continuation\windsurf-cunzhi\target\release\windsurf-cunzhi.exe"
$uiExe = "F:\Trace\TEST\windsurf\Windsurf Infinite Continuation\windsurf-cunzhi\src-tauri\target\release\windsurf-cunzhi-ui.exe"

# 确保目录存在
if (-not (Test-Path $cunzhiDir)) {
    New-Item -ItemType Directory -Path $cunzhiDir -Force | Out-Null
}

# 复制文件
if (Test-Path $mcpExe) {
    Copy-Item $mcpExe -Destination $cunzhiDir -Force
    Write-Host "[OK] Copied windsurf-cunzhi.exe" -ForegroundColor Green
} else {
    Write-Host "[WARN] windsurf-cunzhi.exe not found at: $mcpExe" -ForegroundColor Yellow
}

if (Test-Path $uiExe) {
    Copy-Item $uiExe -Destination $cunzhiDir -Force
    Write-Host "[OK] Copied windsurf-cunzhi-ui.exe" -ForegroundColor Green
} else {
    Write-Host "[WARN] windsurf-cunzhi-ui.exe not found at: $uiExe" -ForegroundColor Yellow
}

Write-Host "[INFO] Cunzhi files prepared in: $cunzhiDir" -ForegroundColor Cyan
