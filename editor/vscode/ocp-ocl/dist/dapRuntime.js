"use strict";

const path = require("path");
const vscode = require("vscode");
const { resolveRuntimePath } = require("./runtimePaths");

const DEBUG_TYPE = "ocp-ocl";

class OclDebugAdapterFactory {
  constructor(dapPath) {
    this.dapPath = dapPath;
  }

  createDebugAdapterDescriptor(session) {
    const cwd = session.workspaceFolder?.uri.fsPath ?? path.dirname(session.configuration.program ?? ".");
    return new vscode.DebugAdapterExecutable(this.dapPath, [], { cwd });
  }
}

class OclDebugConfigurationProvider {
  provideDebugConfigurations(folder) {
    const activePath =
      vscode.window.activeTextEditor?.document.languageId === "ocp-ocl"
        ? vscode.window.activeTextEditor.document.uri.fsPath
        : "${file}";
    return [
      {
        type: DEBUG_TYPE,
        request: "launch",
        name: "OCL Replay Debug",
        program: activePath,
        workspaceFolder: folder?.uri.fsPath ?? "${workspaceFolder}",
        workspaceHash: folder ? stableWorkspaceHash(folder.uri.fsPath) : "workspace",
        runId: "editor-debug",
      },
    ];
  }

  resolveDebugConfiguration(folder, config) {
    if (!vscode.workspace.isTrusted) {
      void vscode.window.showErrorMessage("OCL debug runtime is disabled while the workspace is untrusted.");
      return undefined;
    }
    const activeEditor = vscode.window.activeTextEditor;
    if (!config.program && activeEditor?.document.languageId === "ocp-ocl") {
      config.program = activeEditor.document.uri.fsPath;
    }
    if (!config.program) {
      void vscode.window.showErrorMessage("OCL debug launch requires a saved .ocl file.");
      return undefined;
    }
    config.type = DEBUG_TYPE;
    config.request = "launch";
    config.name = config.name || "OCL Replay Debug";
    config.workspaceFolder = config.workspaceFolder || folder?.uri.fsPath || path.dirname(config.program);
    config.workspaceHash = config.workspaceHash || stableWorkspaceHash(String(config.workspaceFolder));
    config.runId = config.runId || "editor-debug";
    return config;
  }
}

async function startDebugRuntime(context) {
  if (!vscode.workspace.isTrusted) {
    return;
  }

  const dapPath = resolveRuntimePath(context, "ocl-dap.exe");
  if (!dapPath) {
    void vscode.window.setStatusBarMessage("OCL debug runtime missing bundled ocl-dap", 5000);
    return;
  }

  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory(DEBUG_TYPE, new OclDebugAdapterFactory(dapPath)),
  );
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider(DEBUG_TYPE, new OclDebugConfigurationProvider()),
  );
}

function stableWorkspaceHash(input) {
  let hash = 0;
  for (const ch of input) {
    hash = (hash * 131 + ch.charCodeAt(0)) >>> 0;
  }
  return `ws${hash.toString(16)}`;
}

module.exports = {
  startDebugRuntime,
};
