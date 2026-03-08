import * as path from "path";
import * as vscode from "vscode";

import { resolveRuntimePath } from "./runtimePaths";

const DEBUG_TYPE = "ocp";

class OcpDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
  constructor(private readonly dapPath: string) {}

  createDebugAdapterDescriptor(
    session: vscode.DebugSession
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const cwd = session.workspaceFolder?.uri.fsPath ?? path.dirname(session.configuration.program ?? ".");
    return new vscode.DebugAdapterExecutable(this.dapPath, [], { cwd });
  }
}

class OcpDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
  provideDebugConfigurations(
    folder: vscode.WorkspaceFolder | undefined
  ): vscode.ProviderResult<vscode.DebugConfiguration[]> {
    const activePath =
      vscode.window.activeTextEditor?.document.languageId === "ocp"
        ? vscode.window.activeTextEditor.document.uri.fsPath
        : "${file}";
    return [
      {
        type: DEBUG_TYPE,
        request: "launch",
        name: "OCP Replay Debug",
        program: activePath,
        workspaceFolder: folder?.uri.fsPath ?? "${workspaceFolder}",
        workspaceHash: folder ? stableWorkspaceHash(folder.uri.fsPath) : "workspace",
        runId: "editor-debug",
      },
    ];
  }

  resolveDebugConfiguration(
    folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    if (!vscode.workspace.isTrusted) {
      void vscode.window.showErrorMessage("OCP debug runtime is disabled while the workspace is untrusted.");
      return undefined;
    }
    const activeEditor = vscode.window.activeTextEditor;
    if (!config.program && activeEditor?.document.languageId === "ocp") {
      config.program = activeEditor.document.uri.fsPath;
    }
    if (!config.program) {
      void vscode.window.showErrorMessage("OCP debug launch requires a saved .ocp file.");
      return undefined;
    }
    config.type = DEBUG_TYPE;
    config.request = "launch";
    config.name = config.name || "OCP Replay Debug";
    config.workspaceFolder = config.workspaceFolder || folder?.uri.fsPath || path.dirname(config.program);
    config.workspaceHash = config.workspaceHash || stableWorkspaceHash(String(config.workspaceFolder));
    config.runId = config.runId || "editor-debug";
    return config;
  }
}

export async function startDebugRuntime(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.isTrusted) {
    return;
  }

  const dapPath = resolveRuntimePath(context, "ocp-dap.exe");
  if (!dapPath) {
    void vscode.window.setStatusBarMessage("OCP debug runtime missing bundled ocp-dap", 5000);
    return;
  }

  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory(DEBUG_TYPE, new OcpDebugAdapterFactory(dapPath))
  );
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider(DEBUG_TYPE, new OcpDebugConfigurationProvider())
  );
}

function stableWorkspaceHash(input: string): string {
  let hash = 0;
  for (const ch of input) {
    hash = (hash * 131 + ch.charCodeAt(0)) >>> 0;
  }
  return `ws${hash.toString(16)}`;
}
