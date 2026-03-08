"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.startDebugRuntime = startDebugRuntime;
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const runtimePaths_1 = require("./runtimePaths");
const DEBUG_TYPE = "ocp";
class OcpDebugAdapterFactory {
    constructor(dapPath) {
        this.dapPath = dapPath;
    }
    createDebugAdapterDescriptor(session) {
        const cwd = session.workspaceFolder?.uri.fsPath ?? path.dirname(session.configuration.program ?? ".");
        return new vscode.DebugAdapterExecutable(this.dapPath, [], { cwd });
    }
}
class OcpDebugConfigurationProvider {
    provideDebugConfigurations(folder) {
        const activePath = vscode.window.activeTextEditor?.document.languageId === "ocp"
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
    resolveDebugConfiguration(folder, config) {
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
async function startDebugRuntime(context) {
    if (!vscode.workspace.isTrusted) {
        return;
    }
    const dapPath = (0, runtimePaths_1.resolveRuntimePath)(context, "ocp-dap.exe");
    if (!dapPath) {
        void vscode.window.setStatusBarMessage("OCP debug runtime missing bundled ocp-dap", 5000);
        return;
    }
    context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory(DEBUG_TYPE, new OcpDebugAdapterFactory(dapPath)));
    context.subscriptions.push(vscode.debug.registerDebugConfigurationProvider(DEBUG_TYPE, new OcpDebugConfigurationProvider()));
}
function stableWorkspaceHash(input) {
    let hash = 0;
    for (const ch of input) {
        hash = (hash * 131 + ch.charCodeAt(0)) >>> 0;
    }
    return `ws${hash.toString(16)}`;
}
