"use strict";

const fs = require("fs");
const path = require("path");
const vscode = require("vscode");

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
  const configured = vscode.workspace.getConfiguration("ocpOcl").get("serverPath", "").trim();
  const programFiles = process.env.ProgramFiles ?? "C:\\Program Files";
  const candidates = [
    configuredCandidate(configured, binaryName),
    path.join(context.extensionPath, "bin", "win-x64", binaryName),
    path.join(programFiles, "OCP-OCL", binaryName),
    `C:\\Program Files\\OCP-OCL\\${binaryName}`,
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

module.exports = {
  getRuntimeCandidates,
  resolveRuntimePath,
};
