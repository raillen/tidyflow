; Script de Instalação Inno Setup para AutoFlow
#define MyAppName "AutoFlow"
#define MyAppVersion "1.4.1"
#define MyAppPublisher "Raillen Santos"
#define MyAppURL "https://gitlab.com/raillendossantos/autoflow"
#define MyAppExeName "AutoFlow.App.exe"

[Setup]
; AppId único para o instalador
AppId={{9B37E5A1-C1A2-4D3F-B5B6-D4D5E6F7A8B9}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
; Requer privilégios administrativos
PrivilegesRequired=admin
OutputDir=..\release
OutputBaseFilename=AutoFlow_Setup_v{#MyAppVersion}
SetupIconFile=..\src\AutoFlow.App\Assets\app-icon.png
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "portuguese"; MessagesFile: "compiler:Languages\Portuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Aqui apontamos para os binários gerados pelo dotnet publish
Source: "..\src\AutoFlow.App\bin\Release\net10.0\win-x64\publish\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\src\AutoFlow.App\bin\Release\net10.0\win-x64\publish\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; OBS: O ícone do sistema para o instalador precisa estar na pasta de assets

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
