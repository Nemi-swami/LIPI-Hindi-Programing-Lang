; Inno Setup script for a branded LIPI Windows installer.
; Produces LIPI-Setup.exe with the LIPI logo, PATH registration, and an uninstaller.
;
; Build (free — download Inno Setup from https://jrsoftware.org/isdl.php):
;   1. cargo build --release --target x86_64-pc-windows-gnu
;   2. "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" packaging\inno\lipi-setup.iss
;   → dist\LIPI-Setup.exe
;
; (Left as a script rather than a prebuilt .exe because compiling requires the Inno
;  Setup toolchain, which isn't bundled with this repo.)

#define AppName    "LIPI"
#define AppVersion "0.2.0"
#define AppExe     "lipi.exe"

[Setup]
AppId={{9E2C7A10-4B6D-4C2E-9F1A-LIPI00000001}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=LIPI
AppPublisherURL=https://github.com/Nemi-swami/LIPI-Hindi-Programing-Lang
DefaultDirName={autopf}\LIPI
DefaultGroupName=LIPI
DisableProgramGroupPage=yes
OutputDir=..\..\dist
OutputBaseFilename=LIPI-Setup
SetupIconFile=..\lipi.ico
UninstallDisplayIcon={app}\{#AppExe}
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes

[Files]
; The release binary. Adjust the source path to your build output.
Source: "..\..\target\x86_64-pc-windows-gnu\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
; Optional: bundle example programs and the icon.
Source: "..\..\examples\*.swami"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs
Source: "..\lipi.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\LIPI REPL"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\lipi.ico"
Name: "{group}\Uninstall LIPI"; Filename: "{uninstallexe}"

[Tasks]
Name: "addtopath"; Description: "Add LIPI to my PATH"; GroupDescription: "Integration:"

[Registry]
; Append install dir to the user PATH when the task is selected.
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; \
  ValueData: "{olddata};{app}"; Check: NeedsPath('{app}'); Tasks: addtopath

[Run]
Filename: "{app}\{#AppExe}"; Parameters: "--version"; Description: "Verify installation"; Flags: nowait postinstall skipifsilent

[Code]
function NeedsPath(Param: string): Boolean;
var Path: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', Path) then Path := '';
  Result := Pos(';' + ExpandConstant(Param) + ';', ';' + Path + ';') = 0;
end;
