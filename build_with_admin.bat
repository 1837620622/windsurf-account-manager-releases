@echo off
echo Building Windsurf Account Manager...
echo.

:: 编译应用
call npm run tauri build

if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Build successful! Setting administrator privileges...
echo.

:: 设置管理员权限
powershell -ExecutionPolicy Bypass -File "set_admin_manifest.ps1" "src-tauri\target\release\windsurf-account-manager.exe"

echo.
echo Done! The executable now requires administrator privileges.
echo.
echo Output files:
echo - Executable: src-tauri\target\release\windsurf-account-manager.exe
echo - Installer: src-tauri\target\release\bundle\
echo.
pause
