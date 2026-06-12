@echo off
setlocal enabledelayedexpansion
set "LIPI=D:\Rust\cargo\bin\lipi.exe"
set "PROJ=D:\Projects\lipi-lang"
set "EXAMPLES=D:\Projects\lipi-lang\examples"

:menu
cls
echo.
echo  =============================================================
echo   LIPI  --  India's First Devanagari Programming Language
echo   Phases 1-15 Complete  ^|  258 Tests Pass  ^|  Pure Rust
echo  =============================================================
echo.

cd /d "%EXAMPLES%"
if errorlevel 1 ( echo  ERROR: %EXAMPLES% not found & pause & goto :eof )

set COUNT=0
set SWAMI_COUNT=0

:: Count and register all swami files first, then roman
for %%f in (*.swami) do (
    set /a COUNT+=1
    set /a SWAMI_COUNT+=1
    set "FILE_!COUNT!=%%f"
)
for %%f in (*.roman) do (
    set /a COUNT+=1
    set "FILE_!COUNT!=%%f"
)

:: Display swami files
echo   [ .swami ]  Devanagari source files
echo   -------------------------------------------------------------
set IDX=0
for %%f in (*.swami) do (
    set /a IDX+=1
    if !IDX! lss 10 (
        echo     [ !IDX!]  %%f
    ) else (
        echo     [!IDX!]  %%f
    )
)
echo.

:: Display roman files
echo   [ .roman ]  QWERTY transliteration files
echo   -------------------------------------------------------------
set IDX=!SWAMI_COUNT!
for %%f in (*.roman) do (
    set /a IDX+=1
    if !IDX! lss 10 (
        echo     [ !IDX!]  %%f
    ) else (
        echo     [!IDX!]  %%f
    )
)

echo.
echo  =============================================================
echo   N = New File     R = REPL Mode     Q = Quit
echo  =============================================================
echo.
set /p "CHOICE=  >> "

if /i "!CHOICE!"=="Q" goto :eof

if /i "!CHOICE!"=="R" (
    echo.
    echo  Starting REPL...
    echo.
    cd /d "%PROJ%"
    "%LIPI%"
    goto menu
)

if /i "!CHOICE!"=="N" (
    echo.
    set /p "NEWNAME=  New filename (without extension): "
    if "!NEWNAME!"=="" goto menu
    echo.
    echo  Creating: !NEWNAME!.swami
    "%LIPI%" edit "%EXAMPLES%\!NEWNAME!.swami"
    goto menu
)

set "SELECTED=!FILE_%CHOICE%!"
if "!SELECTED!"=="" (
    echo.
    echo  Invalid choice. Try again.
    timeout /t 1 >nul
    goto menu
)

:filemenu
cls
echo.
echo  =============================================================
echo   LIPI  --  !SELECTED!
echo  =============================================================
echo.
echo     [ 1 ]  Run      --  Execute the file
echo     [ 2 ]  Edit     --  Open in LIPI editor (v2)
echo     [ 3 ]  Build    --  Compile to .libc bytecode
echo     [ 4 ]  Roman    --  Run as Roman transliteration
echo     [ 5 ]  Stats    --  File info and line count
echo     [ 6 ]  Copy     --  Duplicate to a new file
echo     [ 0 ]  Back     --  Return to file list
echo.
echo  =============================================================
echo.
set /p "ACTION=  Action >> "

if "!ACTION!"=="0" goto menu

if "!ACTION!"=="1" (
    cls
    echo.
    echo  =============================================================
    echo   Running: !SELECTED!
    echo  =============================================================
    echo.
    cd /d "%PROJ%"
    "%LIPI%" "%EXAMPLES%\!SELECTED!"
    echo.
    echo  =============================================================
    pause
    goto menu
)

if "!ACTION!"=="2" (
    echo.
    echo  Opening editor: !SELECTED!
    "%LIPI%" edit "%EXAMPLES%\!SELECTED!"
    goto menu
)

if "!ACTION!"=="3" (
    cls
    echo.
    echo  =============================================================
    echo   Building: !SELECTED!
    echo  =============================================================
    echo.
    cd /d "%PROJ%"
    "%LIPI%" build "%EXAMPLES%\!SELECTED!"
    echo.
    echo  =============================================================
    pause
    goto menu
)

if "!ACTION!"=="4" (
    cls
    echo.
    echo  =============================================================
    echo   Roman mode: !SELECTED!
    echo  =============================================================
    echo.
    cd /d "%PROJ%"
    "%LIPI%" roman "%EXAMPLES%\!SELECTED!"
    echo.
    echo  =============================================================
    pause
    goto menu
)

if "!ACTION!"=="5" (
    cls
    echo.
    echo  =============================================================
    echo   Stats: !SELECTED!
    echo  =============================================================
    echo.
    for %%F in ("%EXAMPLES%\!SELECTED!") do (
        echo    File  : %%~nxF
        echo    Size  : %%~zF bytes
        echo    Path  : %%~dpF
    )
    echo.
    echo    Lines :
    find /c /v "" "%EXAMPLES%\!SELECTED!"
    echo.
    echo  =============================================================
    pause
    goto filemenu
)

if "!ACTION!"=="6" (
    echo.
    set /p "COPYNAME=  New filename (without extension): "
    if "!COPYNAME!"=="" goto filemenu
    copy "%EXAMPLES%\!SELECTED!" "%EXAMPLES%\!COPYNAME!.swami" >nul
    echo.
    echo  Copied  !SELECTED!  -->  !COPYNAME!.swami
    echo.
    pause
    goto menu
)

goto filemenu
