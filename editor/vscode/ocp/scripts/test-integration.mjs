import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import crypto from "node:crypto";

const extensionRoot = process.cwd();
const repoRoot = path.resolve(extensionRoot, "..", "..", "..");

const requiredFiles = [
  "editor/vscode/ocp/package.json",
  "editor/vscode/ocp/language-configuration.json",
  "editor/vscode/ocp/src/extension.ts",
  "editor/vscode/ocp/src/lspClient.ts",
  "editor/vscode/ocp/src/commands.ts",
  "editor/vscode/ocp/dist/extension.js",
  "editor/vscode/ocp/dist/lspClient.js",
  "editor/vscode/ocp/dist/commands.js",
  "editor/vscode/ocp/syntaxes/ocp.tmLanguage.json",
  "editor/vscode/ocp/icons/ocp-icon-theme.json",
  "editor/vscode/ocp/snippets/ocp.json",
];

for (const rel of requiredFiles) {
  const full = path.join(repoRoot, rel);
  if (!fs.existsSync(full)) {
    throw new Error(`missing required integration file: ${rel}`);
  }
}

const report = {
  schema: "ocp.w19.editor.integration_cli_report.v1",
  status: "PASS",
  command: "pnpm --dir editor/vscode/ocp run test:integration",
  checks: requiredFiles,
  run_manifest_ref: "target/ocp/w19/meta/run_manifest.json",
  run_manifest_sha256: (() => {
    const runManifestPath = path.join(
      repoRoot,
      "target",
      "ocp",
      "w19",
      "meta",
      "run_manifest.json",
    );
    if (!fs.existsSync(runManifestPath)) {
      throw new Error("missing run manifest: target/ocp/w19/meta/run_manifest.json");
    }
    const bytes = fs.readFileSync(runManifestPath);
    return crypto.createHash("sha256").update(bytes).digest("hex");
  })(),
};

const outPath = path.join(
  repoRoot,
  "target",
  "ocp",
  "w19",
  "rc",
  "vscode_integration_cli_report.json",
);
fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");

console.log("v19 integration smoke PASS");
