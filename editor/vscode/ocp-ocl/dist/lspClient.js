"use strict";

const vscode = require("vscode");

async function startLanguageClient(_context) {
  if (!vscode.workspace.isTrusted) {
    await vscode.commands.executeCommand("setContext", "ocpOcl.workspaceTrusted", false);
    return;
  }

  await vscode.commands.executeCommand("setContext", "ocpOcl.workspaceTrusted", true);
  void vscode.window.setStatusBarMessage("OCL Language Server: contract-locked (gate 19-B)", 4000);
}

module.exports = {
  startLanguageClient,
};
