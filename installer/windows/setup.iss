; Inno Setup script — FTP Simulator slim installer
; Requiert Docker Desktop pour gérer PostgreSQL localement.

#define AppName "FTP Simulator"
#define AppVersion "1.0.0"
#define AppPublisher "FTP Simulator Team"
#define AppURL "http://localhost:3000"
#define AppExeName "ftp-tray.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
DefaultDirName={autopf}\FtpSimulator
DefaultGroupName={#AppName}
OutputDir=..\..\dist
OutputBaseFilename=FtpSimulator-{#AppVersion}-windows-x64
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
CloseApplications=yes

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Files]
; Binaires applicatifs
Source: "..\..\target\release\ftp-backend.exe";  DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\release\ftp-tray.exe";     DestDir: "{app}"; Flags: ignoreversion
; Compose file pour la DB Docker
Source: "..\docker-compose.prod.yml";             DestDir: "{commonappdata}\FtpSimulator"; Flags: ignoreversion

[Dirs]
Name: "{commonappdata}\FtpSimulator"
Name: "{commonappdata}\FtpSimulator\logs"

[Icons]
Name: "{group}\{#AppName}";           Filename: "{app}\{#AppExeName}"
Name: "{commondesktop}\{#AppName}";   Filename: "{app}\{#AppExeName}"; Tasks: desktopicon
Name: "{group}\Désinstaller {#AppName}"; Filename: "{uninstallexe}"

[Tasks]
Name: desktopicon; Description: "Créer une icône sur le bureau"; Flags: unchecked
Name: autostart;   Description: "Lancer au démarrage de Windows"; Flags: unchecked

[Run]
; Étape 1 : Démarrer la base PostgreSQL via Docker Compose
Filename: "docker"; Parameters: "compose -f ""{commonappdata}\FtpSimulator\docker-compose.prod.yml"" up -d --wait"; \
  Description: "Démarrage de PostgreSQL (Docker)..."; \
  Flags: runhidden waituntilterminated

; Étape 2 : Lancer l'interface
Filename: "{app}\{#AppExeName}"; Description: "Lancer FTP Simulator"; \
  Flags: nowait postinstall skipifsilent

[Registry]
; Démarrage automatique du tray au démarrage de session
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; \
  ValueType: string; ValueName: "FtpSimulatorTray"; \
  ValueData: """{app}\ftp-tray.exe"""; \
  Flags: uninsdeletevalue; Tasks: autostart

[UninstallRun]
; Arrêter le conteneur à la désinstallation (données conservées dans le volume Docker)
Filename: "docker"; Parameters: "compose -f ""{commonappdata}\FtpSimulator\docker-compose.prod.yml"" stop"; \
  Flags: runhidden

[Code]

// ── Vérification de Docker Desktop ──────────────────────────────────────────

function DockerDesktopInstalled: Boolean;
var
  Path: string;
begin
  // Docker Desktop installe docker.exe dans %ProgramFiles%\Docker\Docker\resources\bin\
  Result := FileExists(ExpandConstant('{pf}\Docker\Docker\resources\bin\docker.exe'))
         or RegQueryStringValue(HKLM, 'SOFTWARE\Docker Inc.\Docker Desktop', 'InstallPath', Path);
end;

function InitializeSetup: Boolean;
var
  Res: Integer;
begin
  Result := True;
  if not DockerDesktopInstalled then
  begin
    Res := MsgBox(
      'Docker Desktop n''est pas installé.' + #13#10 + #13#10 +
      'FTP Simulator utilise Docker pour gérer PostgreSQL localement.' + #13#10 +
      'Vous devez installer Docker Desktop avant de continuer.' + #13#10 + #13#10 +
      'Téléchargez Docker Desktop sur : https://www.docker.com/products/docker-desktop/' + #13#10 + #13#10 +
      'Cliquez OK pour ouvrir la page de téléchargement, ou Annuler pour quitter.',
      mbConfirmation, MB_OKCANCEL
    );
    if Res = IDOK then
      ShellExec('open', 'https://www.docker.com/products/docker-desktop/', '', '', SW_SHOW, ewNoWait, Res);
    Result := False;  // Bloquer l'installation
  end;
end;

// ── Génération du fichier d'environnement ────────────────────────────────────

function GeneratePassword(Len: Integer): string;
var
  Chars: string;
  i, Idx: Integer;
begin
  Chars := 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#%';
  Result := '';
  for i := 1 to Len do
  begin
    Idx := Random(Length(Chars)) + 1;
    Result := Result + Copy(Chars, Idx, 1);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  DbPass, EnvContent, PwdFile: string;
  AppData: string;
begin
  if CurStep = ssPostInstall then
  begin
    AppData := ExpandConstant('{commonappdata}\FtpSimulator');

    // Générer et sauvegarder le mot de passe DB
    PwdFile := AppData + '\db_password';
    if not FileExists(PwdFile) then
    begin
      DbPass := GeneratePassword(32);
      SaveStringToFile(PwdFile, DbPass, False);
    end else
      LoadStringFromFile(PwdFile, DbPass);

    // Écrire le fichier d'environnement du backend
    if not FileExists(AppData + '\env') then
    begin
      EnvContent :=
        'DATABASE_URL=postgresql://ftp_simulator:' + DbPass + '@127.0.0.1:5432/ftp_simulator' + #13#10 +
        'LISTEN_ADDR=127.0.0.1:3000' + #13#10 +
        'RUST_LOG=warn' + #13#10;
      SaveStringToFile(AppData + '\env', EnvContent, False);
    end;
  end;
end;
