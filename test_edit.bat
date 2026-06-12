@echo off
set PATH=D:\Rust\cargo\bin;D:\msys64\mingw64\bin;%PATH%
cd /d D:\Projects\lipi-lang\examples
echo Running: lipi edit hello.swami
echo.
lipi edit hello.swami
echo.
echo Exit code: %errorlevel%
echo.
pause
