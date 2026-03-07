import * as vscode from "vscode";
import { registerCommands } from "./commands";
import { startDebugRuntime } from "./dapRuntime";
import { startLanguageClient } from "./lspClient";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  registerCommands(context);
  await startLanguageClient(context);
  await startDebugRuntime(context);
}

export async function deactivate(): Promise<void> {
  // Runtime disposables are attached to the extension context.
}
