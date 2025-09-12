@echo off
echo Cleaning up git configuration...

REM Remove the proxy configuration if it exists
git config --unset url."https://gh-proxy.com/https://github.com/".insteadOf 2>nul
if %errorlevel% equ 0 (
    echo Git proxy configuration removed
) else (
    echo No git proxy configuration found
)

REM Check current git config
echo.
echo Current git configuration:
git config --list | findstr insteadOf
if %errorlevel% neq 0 (
    echo No insteadOf configurations found
)

echo.
echo Cleanup completed
pause