@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

REM Enable ANSI color support
reg add HKCU\Console /v VirtualTerminalLevel /t REG_DWORD /d 1 /f >nul 2>&1

REM Environment variables will be automatically cleaned up when script exits

echo =============================================
echo    Claude Config Manager Build Script
echo =============================================
echo.

REM Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    powershell -Command "Write-Host 'Warning: Not running as administrator. This may cause permission issues.' -ForegroundColor Red"
    powershell -Command "Write-Host 'If build fails, try running as administrator.' -ForegroundColor Red"
    echo.
)

REM Check basic environment
echo [1/8] Checking build environment...
where node >nul 2>nul || (
    echo Error: Node.js not found, please install Node.js first
    echo Download: https://registry.npmmirror.com/binary.html?path=node/
    pause & exit /b 1
)

where cargo >nul 2>nul || (
    echo Error: Rust/Cargo not found, please install Rust first  
    echo Download: https://forge.rust-lang.org/infra/channel-layout.html#mirrors
    pause & exit /b 1
)

REM Configure Rust mirror
echo [2/8] Configuring Rust mirror...
if not exist "%USERPROFILE%\.cargo\config.toml" (
    if not exist "%USERPROFILE%\.cargo" mkdir "%USERPROFILE%\.cargo"
    (
        echo [source.crates-io]
        echo replace-with = 'rsproxy-sparse'
        echo.
        echo [source.rsproxy-sparse]
        echo registry = "sparse+https://rsproxy.cn/index/"
        echo.
        echo [registries.rsproxy]
        echo index = "https://rsproxy.cn/crates.io-index"
        echo.
        echo [net]
        echo retry = 2
        echo git-fetch-with-cli = true
        echo.
        echo [http]
        echo timeout = 60
    ) > "%USERPROFILE%\.cargo\config.toml"
    echo Rust mirror configured
) else (
    echo Rust mirror already exists
)

REM Configure npm mirror
echo [3/8] Configuring npm mirror...
npm config get registry 2>nul | findstr "npmmirror.com" >nul
if %errorlevel% neq 0 (
    npm config set registry https://registry.npmmirror.com/
    echo npm mirror configured
) else (
    echo npm mirror already configured
)

REM Install dependencies
echo [4/8] Installing project dependencies...
if not exist "node_modules" (
    echo Installing npm dependencies...
    npm install || (
        echo npm dependency installation failed
        pause & exit /b 1
    )
    echo npm dependencies installed
) else (
    echo npm dependencies already exist
)

REM Pre-download WiX tools
echo [5/8] Pre-downloading WiX tools...
set "CACHE_DIR=%USERPROFILE%\.tauri-cache"
set "WIX_FILE=%CACHE_DIR%\wix314-binaries.zip"
set "EXTRACT_DIR=%CACHE_DIR%\wix314"

if not exist "%WIX_FILE%" (
    echo Creating cache directory...
    if not exist "%CACHE_DIR%" mkdir "%CACHE_DIR%"
    
    echo Downloading WiX tools from Chinese mirror...
    curl -L -o "%WIX_FILE%" "https://gh-proxy.com/https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip" --connect-timeout 30 --max-time 300
    if %errorlevel% neq 0 (
        echo Mirror failed, trying original source...
        curl -L -o "%WIX_FILE%" "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip" --connect-timeout 30 --max-time 300
        if %errorlevel% neq 0 (
            echo Failed to download WiX tools
            pause & exit /b 1
        )
    )
    echo WiX tools downloaded successfully
) else (
    echo WiX tools already cached
)

if not exist "%EXTRACT_DIR%" (
    echo Extracting WiX tools...
    mkdir "%EXTRACT_DIR%"
    powershell -Command "Expand-Archive -Path '%WIX_FILE%' -DestinationPath '%EXTRACT_DIR%' -Force" 2>nul
    if %errorlevel% neq 0 (
        echo Extraction failed, but continuing with build...
    ) else (
        echo WiX tools extracted successfully
    )
)

REM Configure build environment
echo [6/8] Configuring build environment...

REM WiX tool download mirror
set "WIX_MIRROR=https://gh-proxy.com/https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"

REM Set all possible WiX environment variables
set "WIX3_DOWNLOAD_URL=%WIX_MIRROR%"
set "TAURI_WIX3_DOWNLOAD_URL=%WIX_MIRROR%"
set "TAURI_BUNDLE_WIX_DOWNLOAD_URL=%WIX_MIRROR%"
set "WIX_DOWNLOAD_URL=%WIX_MIRROR%"
set "TAURI_WIX_DOWNLOAD_URL=%WIX_MIRROR%"
set "TAURI_BUNDLE_WIX3_DOWNLOAD_URL=%WIX_MIRROR%"

REM Point to local WiX if available
if exist "%EXTRACT_DIR%\candle.exe" (
    set "WIX=%EXTRACT_DIR%"
    set "PATH=%EXTRACT_DIR%;%PATH%"
    echo Using local WiX tools: %EXTRACT_DIR%
)

REM Cargo network optimization
set "CARGO_HTTP_TIMEOUT=120"
set "CARGO_NET_RETRY=3"
set "CARGO_HTTP_MULTIPLEXING=false"

REM Configure proxy URLs using environment variables only
set "HTTPS_PROXY="
set "HTTP_PROXY="
set "GIT_CONFIG_GLOBAL="

echo Build environment configured
echo WiX mirror: %WIX_MIRROR%
echo Using environment variables for proxy configuration
echo.

REM Clean up before build
echo [7/8] Cleaning up previous builds...

REM Kill any existing processes that might be using the exe
echo Stopping any running instances...
taskkill /F /IM claude-config-manager.exe >nul 2>&1
timeout /T 2 /NOBREAK >nul

REM Clean previous build
echo Cleaning previous build artifacts...
if exist "src-tauri\target\release\claude-config-manager.exe" (
    del /F /Q "src-tauri\target\release\claude-config-manager.exe" >nul 2>&1
)
if exist "src-tauri\target\release\bundle" (
    rmdir /S /Q "src-tauri\target\release\bundle" >nul 2>&1
)

REM Start build
echo [8/8] Starting build...
echo This may take several minutes, please wait...
echo.

REM Set temporary environment variables for this build only
set "CARGO_NET_GIT_FETCH_WITH_CLI=true"
set "CARGO_HTTP_CAINFO="

REM Configure GitHub mirror for cargo dependencies (temporary)
set "CARGO_REGISTRIES_CRATES_IO_INDEX=sparse+https://rsproxy.cn/index/"

REM Run build with all environment variables in current session
echo Using temporary proxy configuration for this build session only...
npm run tauri build

REM Environment variables automatically cleaned up when script exits

REM Check build result
if %errorlevel% equ 0 (
    echo.
    echo ==========================================
    echo Build successful!
    echo ==========================================
    echo.
    echo Build artifacts location:
    echo     Windows MSI: src-tauri\target\release\bundle\msi\
    echo     NSIS Installer: src-tauri\target\release\bundle\nsis\
    echo     Executable: src-tauri\target\release\claude-config-manager.exe
    echo.
    echo You can find the installer in the above directories
    
    REM List actual files created
    if exist "src-tauri\target\release\bundle\msi\" (
        echo.
        echo MSI files:
        dir "src-tauri\target\release\bundle\msi\*.msi" /B 2>nul
    )
    if exist "src-tauri\target\release\bundle\nsis\" (
        echo.
        echo NSIS files:
        dir "src-tauri\target\release\bundle\nsis\*.exe" /B 2>nul
    )
) else (
    echo.
    echo ==========================================
    echo Build failed!
    echo ==========================================
    echo.
    echo Troubleshooting suggestions:
    powershell -Command "Write-Host '1. Run this script as administrator' -ForegroundColor Red"
    echo 2. Check network connection
    echo 3. Close any running instances of the application
    echo 4. Clear cache: rmdir /s /q node_modules ^&^& npm install
    echo 5. Clear Rust cache: cargo clean
    echo 6. Check Rust toolchain: rustup update
    echo 7. Delete cache and retry: rmdir /s /q "%CACHE_DIR%"
    echo 8. Restart your computer if permission errors persist
    echo.
    echo If the problem persists, please check the error messages above
)

echo.
pause