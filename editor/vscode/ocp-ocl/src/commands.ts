import * as fs from "fs";
import * as path from "path";
import { execFile } from "child_process";
import { promisify } from "util";
import * as vscode from "vscode";
import { resolveRuntimePath } from "./runtimePaths";

const execFileAsync = promisify(execFile);
const OUTPUT_CHANNEL_NAME = "OCP OCL";

function createOutputChannel(context: vscode.ExtensionContext): vscode.OutputChannel {
  const output = vscode.window.createOutputChannel(OUTPUT_CHANNEL_NAME);
  context.subscriptions.push(output);
  return output;
}

function requireWorkspaceRoot(): string {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    throw new Error("Open a workspace folder before running OCL editor commands.");
  }
  return folder.uri.fsPath;
}

function quoteArg(value: string): string {
  return /\s/.test(value) ? `"${value}"` : value;
}

function getLaneProfile(): string {
  return vscode.workspace.getConfiguration("ocpOcl").get<string>("laneProfile", "locked_v071");
}

function resolveCliPath(context: vscode.ExtensionContext): string {
  const cli = resolveRuntimePath(context, "ocl.exe");
  if (cli) {
    return cli;
  }
  throw new Error(
    "Could not resolve OCL CLI. Expected bundled CLI in the VSIX or a valid ocpOcl.serverPath override."
  );
}

async function requireTrustedWorkspace(): Promise<void> {
  if (!vscode.workspace.isTrusted) {
    throw new Error("OCL runtime commands are disabled while the workspace is untrusted.");
  }
}

async function runCli(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel,
  args: string[],
  cwd: string,
  allowNonZero = false
): Promise<{ stdout: string; stderr: string }> {
  const cli = resolveCliPath(context);
  output.show(true);
  output.appendLine(`> ${quoteArg(cli)} ${args.map(quoteArg).join(" ")}`);

  try {
    const result = await execFileAsync(cli, args, { cwd });
    if (result.stdout.trim().length > 0) {
      output.appendLine(result.stdout.trimEnd());
    }
    if (result.stderr.trim().length > 0) {
      output.appendLine(result.stderr.trimEnd());
    }
    return result;
  } catch (error) {
    const err = error as NodeJS.ErrnoException & { stdout?: string; stderr?: string };
    if (err.stdout?.trim().length) {
      output.appendLine(err.stdout.trimEnd());
    }
    if (err.stderr?.trim().length) {
      output.appendLine(err.stderr.trimEnd());
    }
    if (allowNonZero) {
      return {
        stdout: err.stdout ?? "",
        stderr: err.stderr ?? "",
      };
    }
    throw new Error(err.stderr?.trim() || err.stdout?.trim() || err.message);
  }
}

async function openFile(filePath: string): Promise<void> {
  const document = await vscode.workspace.openTextDocument(filePath);
  await vscode.window.showTextDocument(document, { preview: false });
}

function ensureParentDir(filePath: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function latestAuditPath(workspaceRoot: string): string | null {
  const artifactsRoot = path.join(workspaceRoot, ".ocl_artifacts");
  if (!fs.existsSync(artifactsRoot)) {
    return null;
  }

  const candidates = fs
    .readdirSync(artifactsRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => {
      const auditPath = path.join(artifactsRoot, entry.name, "audit.jsonl");
      if (!fs.existsSync(auditPath)) {
        return null;
      }
      return {
        auditPath,
        mtimeMs: fs.statSync(auditPath).mtimeMs,
      };
    })
    .filter((value): value is { auditPath: string; mtimeMs: number } => value !== null)
    .sort((left, right) => right.mtimeMs - left.mtimeMs);

  return candidates[0]?.auditPath ?? null;
}

async function formatDocument(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await requireTrustedWorkspace();
  const workspaceRoot = requireWorkspaceRoot();
  const editor = vscode.window.activeTextEditor;
  if (editor?.document.isDirty) {
    await editor.document.save();
  }
  await runCli(context, output, ["fmt", workspaceRoot], workspaceRoot);
  void vscode.window.showInformationMessage("OCL format completed.");
}

async function checkWorkspace(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await requireTrustedWorkspace();
  const workspaceRoot = requireWorkspaceRoot();
  const args = ["check", workspaceRoot];
  if (getLaneProfile() !== "quarantine") {
    args.push("--locked");
  }
  await runCli(context, output, args, workspaceRoot);
  void vscode.window.showInformationMessage("OCL check completed.");
}

async function openDoctorReport(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await requireTrustedWorkspace();
  const workspaceRoot = requireWorkspaceRoot();
  const reportPath = path.join(workspaceRoot, "target", "ocl", "editor", "doctor_report.json");
  ensureParentDir(reportPath);
  const result = await runCli(
    context,
    output,
    ["doctor", workspaceRoot, "--out", reportPath],
    workspaceRoot,
    true
  );
  if (!fs.existsSync(reportPath)) {
    throw new Error(result.stderr.trim() || "OCL doctor did not produce a report.");
  }
  await openFile(reportPath);
  if (result.stderr.trim().length > 0) {
    void vscode.window.showWarningMessage("OCL doctor reported findings. Report opened.");
  }
}

async function openFixPlan(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await requireTrustedWorkspace();
  const workspaceRoot = requireWorkspaceRoot();
  const planPath = path.join(workspaceRoot, "target", "ocl", "editor", "permission_fix_plan.json");
  ensureParentDir(planPath);
  await runCli(context, output, ["fix", "--plan", workspaceRoot, "--out", planPath], workspaceRoot);
  await openFile(planPath);
}

async function openDebugTrace(output: vscode.OutputChannel): Promise<void> {
  const workspaceRoot = requireWorkspaceRoot();
  const auditPath = latestAuditPath(workspaceRoot);
  if (!auditPath) {
    throw new Error("No audit.jsonl artifact found. Run `ocl run` first.");
  }
  output.show(true);
  output.appendLine(`Opening latest audit trace: ${auditPath}`);
  await openFile(auditPath);
}

async function openBudgetAnalyze(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await requireTrustedWorkspace();
  const workspaceRoot = requireWorkspaceRoot();
  const auditPath = latestAuditPath(workspaceRoot);
  if (!auditPath) {
    throw new Error("No audit.jsonl artifact found. Run `ocl run` first.");
  }
  const reportPath = path.join(workspaceRoot, "target", "ocl", "editor", "budget_report.json");
  ensureParentDir(reportPath);
  const result = await runCli(
    context,
    output,
    ["budget", "analyze", auditPath, "--json"],
    workspaceRoot
  );
  const rendered = result.stdout.trim();
  if (rendered.length === 0) {
    throw new Error("OCL budget analyze did not return JSON output.");
  }
  fs.writeFileSync(reportPath, rendered, "utf8");
  await openFile(reportPath);
}

async function openCassetteReport(output: vscode.OutputChannel): Promise<void> {
  await openDebugTrace(output);
}

async function openContractMismatch(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await openDoctorReport(context, output);
}

async function openDiffReport(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await openDoctorReport(context, output);
}

async function applyPatchGuidance(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  await openFixPlan(context, output);
  void vscode.window.showWarningMessage(
    "OCL apply patch requires explicit approval. Review the generated fix plan and run `ocl fix --apply ...` from the terminal."
  );
}

async function runCommand(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel,
  command: () => Promise<void>
): Promise<void> {
  try {
    await command();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    output.show(true);
    output.appendLine(`ERROR: ${message}`);
    void vscode.window.showErrorMessage(message);
  }
}

export function registerCommands(context: vscode.ExtensionContext): void {
  const output = createOutputChannel(context);

  const handlers: Array<[string, () => Promise<void>]> = [
    ["ocpOcl.formatDocument", () => formatDocument(context, output)],
    ["ocpOcl.checkWorkspace", () => checkWorkspace(context, output)],
    ["ocpOcl.openDoctorReport", () => openDoctorReport(context, output)],
    ["ocpOcl.openFixPlan", () => openFixPlan(context, output)],
    ["ocpOcl.openDebugTrace", () => openDebugTrace(output)],
    ["ocpOcl.codeAction.permissionFixPlan", () => openFixPlan(context, output)],
    ["ocpOcl.codeAction.openDiff", () => openDiffReport(context, output)],
    ["ocpOcl.codeAction.applyPatch", () => applyPatchGuidance(context, output)],
    ["ocpOcl.codeAction.openBudgetAnalyze", () => openBudgetAnalyze(context, output)],
    ["ocpOcl.codeAction.openCassetteReport", () => openCassetteReport(output)],
    ["ocpOcl.codeAction.openContractMismatch", () => openContractMismatch(context, output)],
  ];

  for (const [commandId, handler] of handlers) {
    const disposable = vscode.commands.registerCommand(commandId, async () => {
      await runCommand(context, output, handler);
    });
    context.subscriptions.push(disposable);
  }
}
