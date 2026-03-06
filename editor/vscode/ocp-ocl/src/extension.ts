import * as vscode from "vscode";
import { registerCommands } from "./commands";
import { startLanguageClient } from "./lspClient";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  registerCommands(context);
  await startLanguageClient(context);
}

export async function deactivate(): Promise<void> {
  // No-op for gate 19-A. LSP lifecycle is locked in later gates.
}
