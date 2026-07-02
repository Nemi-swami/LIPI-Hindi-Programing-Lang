[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=0
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=%InstallPrompt%
DisplayLicense=%DisplayLicense%
FinishMessage=%FinishMessage%
TargetName=%TargetName%
FriendlyName=%FriendlyName%
AppLaunched=%AppLaunched%
PostInstallCmd=%PostInstallCmd%
AdminQuietInstCmd=%AdminQuietInstCmd%
UserQuietInstCmd=%UserQuietInstCmd%
SourceFiles=SourceFiles
[Strings]
InstallPrompt=Install LIPI (a Devanagari-syntax programming language)?
DisplayLicense=
FinishMessage=LIPI installed. Open a NEW terminal and run:  lipi
TargetName=D:\Projects\lipi-lang\dist\LIPI-Setup.exe
FriendlyName=LIPI Setup
AppLaunched=cmd /c install_local.bat
PostInstallCmd=<None>
AdminQuietInstCmd=
UserQuietInstCmd=
FILE0="install_local.bat"
FILE1="lipi.exe"
[SourceFiles]
SourceFiles0=D:\Projects\lipi-lang\packaging\stage\
[SourceFiles0]
%FILE0%=
%FILE1%=
