@echo off
chcp 65001 >nul
title LIPI Playground
setlocal enabledelayedexpansion
cd /d "%~dp0"

:: ── Rust environment ────────────────────────────────────────────────────────
set RUSTUP_HOME=D:\Rust\rustup
set CARGO_HOME=D:\Rust\cargo
set PATH=D:\Rust\cargo\bin;D:\msys64\mingw64\bin;%PATH%

:: Absolute path to web directory (avoids 404 from wrong working dir)
set WEB_DIR=%~dp0web

echo.
echo  ==========================================
echo   LIPI  -  India's Programming Language
echo   WASM Browser Playground
echo  ==========================================
echo.

:: ── Optional: force rebuild ─────────────────────────────────────────────────
set REBUILD=0
if /i "%~1"=="rebuild" set REBUILD=1

:: ── Step 1: WASM build ──────────────────────────────────────────────────────
if "%REBUILD%"=="1" (
    echo  [1/3] Building WASM (forced)...
    goto :do_build
)
if not exist "%WEB_DIR%\pkg\lipi_bg.wasm" (
    echo  [1/3] First run - compiling WASM (this takes ~30 seconds)...
    goto :do_build
)
echo  [1/3] WASM is ready.  (run  play.bat rebuild  to recompile)
goto :find_python

:do_build
wasm-pack build --target web --out-dir "%WEB_DIR%\pkg" --features wasm
if errorlevel 1 (
    echo.
    echo  [ERROR] WASM build failed! See errors above.
    pause
    exit /b 1
)
echo  [1/3] WASM build successful!

:: ── Step 2: Find Python ─────────────────────────────────────────────────────
:find_python
echo  [2/3] Looking for Python...

set PY=
where python  >nul 2>&1 && set PY=python
if "!PY!"=="" (
    where python3 >nul 2>&1 && set PY=python3
)
if "!PY!"=="" (
    echo.
    echo  [ERROR] Python not found.
    echo  Please install Python from https://python.org
    pause
    exit /b 1
)
echo  [2/3] Found: !PY!

:: ── Step 3: Start server and open browser ───────────────────────────────────
echo  [3/3] Starting server...
echo.
echo  Playground : http://localhost:8080
echo  Stop       : press Ctrl+C in this window
echo.

:: Open browser after 2-second delay so server is ready
start "" cmd /c "timeout /t 2 /nobreak >nul & start http://localhost:8080/index.html"

:: Start HTTP server with absolute path (blocks until Ctrl+C)
!PY! -m http.server 8080 --directory "%WEB_DIR%"

endlocal
