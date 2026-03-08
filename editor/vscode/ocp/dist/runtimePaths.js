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
exports.getRuntimeCandidates = getRuntimeCandidates;
exports.resolveRuntimePath = resolveRuntimePath;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
function configuredCandidate(configured, binaryName) {
    if (configured.length === 0) {
        return "";
    }
    if (configured.toLowerCase().endsWith(".exe")) {
        const configuredBase = path.basename(configured).toLowerCase();
        if (configuredBase === binaryName.toLowerCase()) {
            return configured;
        }
        return path.join(path.dirname(configured), binaryName);
    }
    return path.join(configured, binaryName);
}
function getRuntimeCandidates(context, binaryName) {
    const configured = vscode.workspace.getConfiguration("ocp").get("serverPath", "").trim();
    const programFiles = process.env.ProgramFiles ?? "C:\\Program Files";
    const candidates = [
        configuredCandidate(configured, binaryName),
        path.join(context.extensionPath, "bin", "win-x64", binaryName),
        path.join(programFiles, "OCP", binaryName),
        `C:\\Program Files\\OCP\\${binaryName}`,
    ];
    return candidates.filter((item, index) => item.length > 0 && candidates.indexOf(item) === index);
}
function resolveRuntimePath(context, binaryName) {
    for (const candidate of getRuntimeCandidates(context, binaryName)) {
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }
    return null;
}
