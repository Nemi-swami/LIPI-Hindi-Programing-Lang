@echo off
set RUSTUP_HOME=D:\Rust\rustup
set CARGO_HOME=D:\Rust\cargo
set PATH=D:\Rust\cargo\bin;D:\msys64\mingw64\bin;%PATH%

cd /d D:\Projects\lipi-lang

echo.
echo  LIPI — Building...
echo  ══════════════════════════════════════
cargo build --target x86_64-pc-windows-gnu
if errorlevel 1 (
    echo.
    echo  [ERROR] Build failed!
    pause
    exit /b 1
)

echo.
echo  Installing lipi.exe to D:\Rust\cargo\bin\
copy /Y "target\x86_64-pc-windows-gnu\debug\lipi.exe" "D:\Rust\cargo\bin\lipi.exe" >nul

echo.
echo  Done! You can now use from anywhere:
echo.
echo    lipi foo.swami          run a file
echo    lipi edit foo.swami     open editor
echo    lipi build foo.swami    compile to .libc
echo    lipi run foo.libc       run compiled
echo    lipi                    REPL
echo.
pause
