@echo off
setlocal
set "DEST=%LOCALAPPDATA%\LIPI\bin"
if not exist "%DEST%" mkdir "%DEST%"
copy /Y "%~dp0lipi.exe" "%DEST%\lipi.exe" >nul
echo %PATH% | find /I "%DEST%" >nul
if errorlevel 1 (
  setx PATH "%PATH%;%DEST%" >nul
  echo Added %DEST% to your PATH ^(restart the terminal^).
)
echo.
echo LIPI installed to %DEST%\lipi.exe
echo Try:  lipi   ^(REPL^)   or write a .swami file and run  lipi file.swami
echo.
pause
endlocal
