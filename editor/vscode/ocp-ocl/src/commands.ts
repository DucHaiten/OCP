import * as vscode from "vscode";

const COMMANDS = [
  "ocpOcl.formatDocument",
  "ocpOcl.checkWorkspace",
  "ocpOcl.openDoctorReport",
  "ocpOcl.openFixPlan",
  "ocpOcl.openDebugTrace"
];

export function registerCommands(context: vscode.ExtensionContext): void {
  for (const command of COMMANDS) {
    const disposable = vscode.commands.registerCommand(command, async () => {
      void vscode.window.showInformationMessage(
        `Command '${command}' is registered in gate 19-A and will be fully wired in later gates.`
      );
    });
    context.subscriptions.push(disposable);
  }
}
