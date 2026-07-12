; Inno Setup script — one-click installer for the NATIVE LIPI Studio desktop app
; (Electron). Bundles the whole app (Electron runtime + LIPI compiler), installs to
; Program Files, creates Start-Menu + desktop shortcuts, registers .swami/.roman/.vani
; file associations with the LIPI icon, and an uninstaller. Compile:
;   "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" packaging\inno\lipi-studio-setup.iss
;   → dist\LIPI-Studio-Setup.exe

#define AppName    "LIPI Studio"
#define AppVersion "0.3.0"
#define AppExe     "LIPI Studio.exe"
#define Unpacked   "..\..\desktop\release\win-unpacked"

[Setup]
AppId={{B7E4B1B0-3C2A-4E7D-9A5C-LIPISTUDIO01}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=LIPI
AppPublisherURL=https://github.com/Nemi-swami/LIPI-Hindi-Programing-Lang
DefaultDirName={autopf}\LIPI Studio
DisableProgramGroupPage=yes
DisableDirPage=yes
OutputDir=..\..\dist
OutputBaseFilename=LIPI-Studio-0.3.0-Setup
SetupIconFile=..\..\desktop\assets\icon.ico
UninstallDisplayIcon={app}\{#AppExe}
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
PrivilegesRequired=admin
ChangesAssociations=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"
Name: "assoc";       Description: "Associate .swami / .roman / .vani files with LIPI Studio"; GroupDescription: "File associations:"

[Files]
; The entire built app (Electron runtime, app.asar, bundled lipi.exe, DLLs, locales).
Source: "{#Unpacked}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; A standalone icon for shortcuts + file associations.
Source: "..\..\desktop\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\LIPI Studio"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\icon.ico"
Name: "{autodesktop}\LIPI Studio";  Filename: "{app}\{#AppExe}"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Registry]
; File-type associations — .swami / .roman / .vani open in LIPI Studio and show the
; LIPI icon in Explorer. Written under HKA (per-machine for an admin install).
Root: HKA; Subkey: "Software\Classes\.swami";                     ValueType: string; ValueName: ""; ValueData: "LIPI.Source";  Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.roman";                     ValueType: string; ValueName: ""; ValueData: "LIPI.Source";  Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.vani";                      ValueType: string; ValueName: ""; ValueData: "LIPI.Source";  Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\LIPI.Source";               ValueType: string; ValueName: ""; ValueData: "LIPI Program"; Flags: uninsdeletekey;   Tasks: assoc
Root: HKA; Subkey: "Software\Classes\LIPI.Source\DefaultIcon";    ValueType: string; ValueName: ""; ValueData: "{app}\icon.ico"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\LIPI.Source\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: assoc

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch LIPI Studio"; Flags: nowait postinstall skipifsilent
