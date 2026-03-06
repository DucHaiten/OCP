import * as vscode from "vscode";

export async function startLanguageClient(_context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.isTrusted) {
    await vscode.commands.executeCommand("setContext", "ocpOcl.workspaceTrusted", false);
    return;
  }

  await vscode.commands.executeCommand("setContext", "ocpOcl.workspaceTrusted", true);
  // Gate 19-B locks trust gating + capability handshake. Process launch is expanded in later gates.
  void vscode.window.setStatusBarMessage("OCL Language Server: contract-locked (gate 19-B)", 4000);
}
