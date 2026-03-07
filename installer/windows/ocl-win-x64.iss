#define MyAppName "OCP-OCL"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "OCP-OCL"
#define MyAppExeName "ocl.exe"
#define MyLspExeName "ocl-lsp.exe"
#define MyDapExeName "ocl-dap.exe"
#define MyVsixName "ocp-ocl-vscode-v1.0.0.vsix"
#define MyVsixLogName "vscode-extension-install.log"
#define MyIconFile AddBackslash(SourcePath) + "ocl-installer.ico"

#ifndef StageRoot
  #error "StageRoot define is required"
#endif

#ifndef ReleaseRoot
  #error "ReleaseRoot define is required"
#endif

[Setup]
AppId={{7E9D1D4D-7D3D-4E3A-9E6E-0D79D0F17D10}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\OCP-OCL
DisableDirPage=no
UsePreviousAppDir=no
DefaultGroupName=OCP-OCL
DisableProgramGroupPage=yes
OutputDir={#ReleaseRoot}
OutputBaseFilename=ocl-v1.0.0-setup-win-x64
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
PrivilegesRequired=admin
UninstallDisplayIcon={app}\{#MyAppExeName}
SetupIconFile={#MyIconFile}

[Tasks]
Name: "addtopath"; Description: "Add OCP-OCL CLI to PATH (recommended)"; GroupDescription: "One-time setup options:"; Flags: unchecked
Name: "installvscodeext"; Description: "Install VSCode extension now (recommended)"; GroupDescription: "One-time setup options:"; Flags: checkedonce; Check: HasVSCodeCli

[Files]
Source: "{#StageRoot}\ocl.exe"; DestDir: "{app}"; DestName: "ocl.exe"; Flags: ignoreversion
Source: "{#StageRoot}\ocl-lsp.exe"; DestDir: "{app}"; DestName: "ocl-lsp.exe"; Flags: ignoreversion
Source: "{#StageRoot}\ocl-dap.exe"; DestDir: "{app}"; DestName: "ocl-dap.exe"; Flags: ignoreversion
Source: "{#StageRoot}\INSTALL-NEXT-STEPS.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageRoot}\{#MyVsixName}"; DestDir: "{app}"; Flags: ignoreversion

[Run]
Filename: "{app}\ocl.exe"; Parameters: "--version"; Description: "Verify OCP-OCL CLI installation"; Flags: postinstall nowait skipifsilent
Filename: "{cmd}"; Parameters: "{code:GetVSCodeInstallCommand}"; Description: "Install OCP-OCL VSCode extension"; Flags: postinstall waituntilterminated runasoriginaluser; Tasks: installvscodeext; Check: HasVSCodeCli
Filename: "notepad.exe"; Parameters: """{app}\INSTALL-NEXT-STEPS.txt"""; Description: "Open next steps"; Flags: postinstall skipifsilent unchecked

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath(ExpandConstant('{app}'))

[Code]
function NeedsAddPath(Path: string): Boolean;
var
  CurrentPath: string;
begin
  if not RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', CurrentPath) then
    CurrentPath := '';

  Result := Pos(';' + UpperCase(Path) + ';', ';' + UpperCase(CurrentPath) + ';') = 0;
end;

function ResolveVSCodeCliPath: string;
var
  Candidate: string;
  ResultCode: Integer;
  DetectFile: string;
  DetectOutput: AnsiString;
  LineBreakPos: Integer;
begin
  Result := '';

  Candidate := ExpandConstant('{pf}\Microsoft VS Code\bin\code.cmd');
  if FileExists(Candidate) then
  begin
    Result := Candidate;
    exit;
  end;

  Candidate := ExpandConstant('{localappdata}\Programs\Microsoft VS Code\bin\code.cmd');
  if FileExists(Candidate) then
  begin
    Result := Candidate;
    exit;
  end;

  DetectFile := ExpandConstant('{tmp}\ocl-vscode-cli-path.txt');
  if Exec(ExpandConstant('{cmd}'), '/C where code.cmd > "' + DetectFile + '"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and (ResultCode = 0) then
  begin
    if LoadStringFromFile(DetectFile, DetectOutput) then
    begin
      LineBreakPos := Pos(#13, DetectOutput);
      if LineBreakPos = 0 then
        LineBreakPos := Pos(#10, DetectOutput);
      if LineBreakPos > 0 then
        DetectOutput := Copy(DetectOutput, 1, LineBreakPos - 1);
      DetectOutput := Trim(DetectOutput);
      if (DetectOutput <> '') and FileExists(DetectOutput) then
      begin
        Result := DetectOutput;
        exit;
      end;
    end;
  end;
end;

function GetVSCodeCliPath(Param: string): string;
begin
  Result := ResolveVSCodeCliPath;
end;

function GetVSCodeInstallLogPath(Param: string): string;
begin
  Result := ExpandConstant('{app}\{#MyVsixLogName}');
end;

function GetVSCodeInstallCommand(Param: string): string;
var
  CliPath: string;
  VsixPath: string;
  LogPath: string;
begin
  CliPath := ResolveVSCodeCliPath;
  VsixPath := ExpandConstant('{app}\{#MyVsixName}');
  LogPath := ExpandConstant('{app}\{#MyVsixLogName}');
  Result := '/C ""' + CliPath + '" --install-extension "' + VsixPath + '" --force > "' + LogPath + '" 2>&1"';
end;

function HasVSCodeCli: Boolean;
begin
  Result := ResolveVSCodeCliPath <> '';
end;
