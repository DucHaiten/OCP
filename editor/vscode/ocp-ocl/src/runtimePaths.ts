import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

function configuredCandidate(configured: string, binaryName: string): string {
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

export function getRuntimeCandidates(
  context: vscode.ExtensionContext,
  binaryName: string
): string[] {
  const configured = vscode.workspace.getConfiguration("ocpOcl").get<string>("serverPath", "").trim();
  const programFiles = process.env.ProgramFiles ?? "C:\\Program Files";
  const candidates = [
    configuredCandidate(configured, binaryName),
    path.join(context.extensionPath, "bin", "win-x64", binaryName),
    path.join(programFiles, "OCP-OCL", binaryName),
    `C:\\Program Files\\OCP-OCL\\${binaryName}`,
  ];
  return candidates.filter((item, index) => item.length > 0 && candidates.indexOf(item) === index);
}

export function resolveRuntimePath(
  context: vscode.ExtensionContext,
  binaryName: string
): string | null {
  for (const candidate of getRuntimeCandidates(context, binaryName)) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}
