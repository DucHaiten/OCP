import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import * as vscode from "vscode";

import { resolveRuntimePath } from "./runtimePaths";

const LANGUAGE_ID = "ocp-ocl";
const SEMANTIC_TOKEN_TYPES = [
  "namespace",
  "type",
  "class",
  "enum",
  "interface",
  "struct",
  "typeParameter",
  "parameter",
  "variable",
  "property",
  "enumMember",
  "event",
  "function",
  "method",
  "macro",
  "keyword",
  "modifier",
  "comment",
  "string",
  "number",
  "regexp",
  "operator",
];

type PendingRequest = {
  resolve: (value: any) => void;
  reject: (error: Error) => void;
};

class OclLspRuntime implements vscode.Disposable {
  private readonly diagnostics = vscode.languages.createDiagnosticCollection("ocp-ocl");
  private readonly docSelector: vscode.DocumentSelector = [{ language: LANGUAGE_ID, scheme: "file" }];
  private readonly registrations: vscode.Disposable[] = [];
  private readonly pending = new Map<number, PendingRequest>();
  private readonly openedUris = new Set<string>();
  private readonly semanticLegend = new vscode.SemanticTokensLegend(SEMANTIC_TOKEN_TYPES, []);
  private process: ChildProcessWithoutNullStreams | null = null;
  private nextId = 1;
  private readBuffer = Buffer.alloc(0);
  private disposed = false;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly binaryPath: string
  ) {}

  async start(): Promise<void> {
    this.process = spawn(this.binaryPath, [], {
      cwd: this.workspaceRoot(),
      stdio: "pipe",
    });

    this.process.stdout.on("data", (chunk: Buffer) => {
      this.readBuffer = Buffer.concat([this.readBuffer, chunk]);
      this.drainMessages();
    });
    this.process.stderr.on("data", () => {
      // Stdio is intentionally quiet in the shipped extension.
    });
    this.process.on("exit", () => {
      if (!this.disposed) {
        void vscode.window.showErrorMessage("OCL language runtime stopped unexpectedly.");
      }
    });

    await this.request("initialize", {
      processId: process.pid,
      rootUri: vscode.workspace.workspaceFolders?.[0]?.uri.toString() ?? null,
      capabilities: {},
      workspaceFolders: (vscode.workspace.workspaceFolders ?? []).map((folder) => ({
        uri: folder.uri.toString(),
        name: folder.name,
      })),
    });
    this.notify("initialized", {});
    this.registerProviders();
    await this.syncOpenDocuments();
  }

  dispose(): void {
    this.disposed = true;
    for (const pending of this.pending.values()) {
      pending.reject(new Error("OCL language runtime disposed."));
    }
    this.pending.clear();
    this.diagnostics.dispose();
    for (const registration of this.registrations) {
      registration.dispose();
    }
    if (this.process) {
      try {
        this.notify("shutdown", {});
      } catch {
        // Ignore shutdown path issues during dispose.
      }
      this.process.kill();
      this.process = null;
    }
  }

  private workspaceRoot(): string | undefined {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  }

  private registerProviders(): void {
    this.registrations.push(this.diagnostics);
    this.registrations.push(
      vscode.workspace.onDidOpenTextDocument(async (document) => {
        await this.syncDocument(document, "open");
      })
    );
    this.registrations.push(
      vscode.workspace.onDidChangeTextDocument(async (event) => {
        await this.syncDocument(event.document, "change");
      })
    );
    this.registrations.push(
      vscode.workspace.onDidSaveTextDocument(async (document) => {
        await this.syncDocument(document, "save");
      })
    );
    this.registrations.push(
      vscode.workspace.onDidCloseTextDocument((document) => {
        if (document.languageId !== LANGUAGE_ID || document.uri.scheme !== "file") {
          return;
        }
        this.openedUris.delete(document.uri.toString());
        this.diagnostics.delete(document.uri);
        this.notify("textDocument/didClose", {
          textDocument: { uri: document.uri.toString() },
        });
      })
    );
    this.registrations.push(
      vscode.languages.registerHoverProvider(this.docSelector, {
        provideHover: async (document, position) => {
          const result = await this.request("textDocument/hover", {
            textDocument: { uri: document.uri.toString() },
            position: toLspPosition(position),
          });
          const value = result?.contents?.value;
          if (typeof value !== "string") {
            return undefined;
          }
          return new vscode.Hover(new vscode.MarkdownString(value));
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerCompletionItemProvider(this.docSelector, {
        provideCompletionItems: async (document, position) => {
          const result = await this.request("textDocument/completion", {
            textDocument: { uri: document.uri.toString() },
            position: toLspPosition(position),
          });
          const items = Array.isArray(result?.items) ? result.items : [];
          return items.map((item: any) => {
            const completion = new vscode.CompletionItem(
              String(item.label ?? ""),
              fromCompletionKind(item.kind)
            );
            return completion;
          });
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerDefinitionProvider(this.docSelector, {
        provideDefinition: async (document, position) => {
          const result = await this.request("textDocument/definition", {
            textDocument: { uri: document.uri.toString() },
            position: toLspPosition(position),
          });
          if (!Array.isArray(result)) {
            return undefined;
          }
          return result.map(fromLocation).filter((item): item is vscode.Location => item !== null);
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerReferenceProvider(this.docSelector, {
        provideReferences: async (document, position) => {
          const result = await this.request("textDocument/references", {
            textDocument: { uri: document.uri.toString() },
            position: toLspPosition(position),
            context: { includeDeclaration: true },
          });
          if (!Array.isArray(result)) {
            return [];
          }
          return result.map(fromLocation).filter((item): item is vscode.Location => item !== null);
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerRenameProvider(this.docSelector, {
        provideRenameEdits: async (document, position, newName) => {
          const result = await this.request("textDocument/rename", {
            textDocument: { uri: document.uri.toString() },
            position: toLspPosition(position),
            newName,
          });
          if (!result || typeof result !== "object") {
            return undefined;
          }
          return fromWorkspaceEdit(result);
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerDocumentSymbolProvider(this.docSelector, {
        provideDocumentSymbols: async (document) => {
          const result = await this.request("textDocument/documentSymbol", {
            textDocument: { uri: document.uri.toString() },
          });
          if (!Array.isArray(result)) {
            return [];
          }
          return result
            .map(fromSymbolInformation)
            .filter((item): item is vscode.SymbolInformation => item !== null);
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerWorkspaceSymbolProvider({
        provideWorkspaceSymbols: async (query) => {
          const result = await this.request("workspace/symbol", { query });
          if (!Array.isArray(result)) {
            return [];
          }
          return result
            .map(fromSymbolInformation)
            .filter((item): item is vscode.SymbolInformation => item !== null);
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerDocumentFormattingEditProvider(this.docSelector, {
        provideDocumentFormattingEdits: async (document) => {
          const result = await this.request("textDocument/formatting", {
            textDocument: { uri: document.uri.toString() },
            options: {
              tabSize: 2,
              insertSpaces: true,
            },
          });
          if (!Array.isArray(result)) {
            return [];
          }
          return result.map(fromTextEdit).filter((item): item is vscode.TextEdit => item !== null);
        },
      })
    );
    this.registrations.push(
      vscode.languages.registerCodeActionsProvider(this.docSelector, {
        provideCodeActions: async (document, range, context) => {
          const result = await this.request("textDocument/codeAction", {
            textDocument: { uri: document.uri.toString() },
            range: toLspRange(range),
            context: {
              diagnostics: context.diagnostics.map((diag) => ({
                message: diag.message,
                code: typeof diag.code === "string" ? diag.code : undefined,
              })),
            },
          });
          if (!Array.isArray(result)) {
            return [];
          }
          return result.map((item: any) => {
            const action = new vscode.CodeAction(
              String(item.title ?? "OCL Code Action"),
              vscode.CodeActionKind.QuickFix
            );
            if (item?.command?.command) {
              action.command = {
                command: String(item.command.command),
                title: String(item.command.title ?? item.title ?? "OCL Code Action"),
              };
            }
            return action;
          });
        },
      })
    );

    if (vscode.workspace.getConfiguration("ocpOcl").get<boolean>("enableSemanticTokens", true)) {
      this.registrations.push(
        vscode.languages.registerDocumentSemanticTokensProvider(
          this.docSelector,
          {
            provideDocumentSemanticTokens: async (document) => {
              const result = await this.request("textDocument/semanticTokens/full", {
                textDocument: { uri: document.uri.toString() },
              });
              const data = Array.isArray(result?.data) ? result.data.map((value: any) => Number(value)) : [];
              return new vscode.SemanticTokens(new Uint32Array(data));
            },
          },
          this.semanticLegend
        )
      );
    }
  }

  private async syncOpenDocuments(): Promise<void> {
    const docs = vscode.workspace.textDocuments.filter(
      (document) => document.languageId === LANGUAGE_ID && document.uri.scheme === "file"
    );
    for (const document of docs) {
      await this.syncDocument(document, "open");
    }
  }

  private async syncDocument(
    document: vscode.TextDocument,
    mode: "open" | "change" | "save"
  ): Promise<void> {
    if (document.languageId !== LANGUAGE_ID || document.uri.scheme !== "file") {
      return;
    }
    const uri = document.uri.toString();
    const payload = {
      textDocument: {
        uri,
        languageId: LANGUAGE_ID,
        version: document.version,
        text: document.getText(),
      },
    };
    if (!this.openedUris.has(uri) || mode === "open") {
      this.openedUris.add(uri);
      this.notify("textDocument/didOpen", payload);
      return;
    }
    if (mode === "change") {
      this.notify("textDocument/didChange", {
        textDocument: {
          uri,
          version: document.version,
        },
        contentChanges: [{ text: document.getText() }],
      });
      return;
    }
    this.notify("textDocument/didSave", payload);
  }

  private request(method: string, params: any): Promise<any> {
    if (!this.process) {
      return Promise.reject(new Error("OCL language runtime is not running."));
    }
    const id = this.nextId++;
    const payload = {
      jsonrpc: "2.0",
      id,
      method,
      params,
    };
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.writeMessage(payload);
    });
  }

  private notify(method: string, params: any): void {
    if (!this.process) {
      return;
    }
    this.writeMessage({
      jsonrpc: "2.0",
      method,
      params,
    });
  }

  private writeMessage(payload: any): void {
    if (!this.process) {
      return;
    }
    const body = Buffer.from(JSON.stringify(payload), "utf8");
    this.process.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
    this.process.stdin.write(body);
  }

  private drainMessages(): void {
    while (true) {
      const separator = this.readBuffer.indexOf("\r\n\r\n");
      if (separator === -1) {
        return;
      }
      const headerText = this.readBuffer.slice(0, separator).toString("utf8");
      const match = /Content-Length:\s*(\d+)/i.exec(headerText);
      if (!match) {
        this.readBuffer = Buffer.alloc(0);
        return;
      }
      const length = Number(match[1]);
      const messageStart = separator + 4;
      if (this.readBuffer.length < messageStart + length) {
        return;
      }
      const body = this.readBuffer.slice(messageStart, messageStart + length).toString("utf8");
      this.readBuffer = this.readBuffer.slice(messageStart + length);
      try {
        this.onMessage(JSON.parse(body));
      } catch {
        // Ignore malformed payloads from the bundled runtime.
      }
    }
  }

  private onMessage(message: any): void {
    if (typeof message?.id === "number" && this.pending.has(message.id)) {
      const pending = this.pending.get(message.id);
      this.pending.delete(message.id);
      if (!pending) {
        return;
      }
      if (message.error) {
        pending.reject(new Error(String(message.error.message ?? "OCL LSP request failed.")));
        return;
      }
      pending.resolve(message.result);
      return;
    }

    if (message?.method === "textDocument/publishDiagnostics") {
      const uri = typeof message.params?.uri === "string" ? vscode.Uri.parse(message.params.uri) : null;
      if (!uri) {
        return;
      }
      const diagnostics = Array.isArray(message.params?.diagnostics)
        ? message.params.diagnostics.map(fromDiagnostic).filter((item: vscode.Diagnostic | null): item is vscode.Diagnostic => item !== null)
        : [];
      this.diagnostics.set(uri, diagnostics);
    }
  }
}

export async function startLanguageClient(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.isTrusted) {
    await vscode.commands.executeCommand("setContext", "ocpOcl.workspaceTrusted", false);
    void vscode.window.setStatusBarMessage("OCL editor runtime disabled: workspace is untrusted", 4000);
    return;
  }

  await vscode.commands.executeCommand("setContext", "ocpOcl.workspaceTrusted", true);
  const lspPath = resolveRuntimePath(context, "ocl-lsp.exe");
  if (!lspPath) {
    void vscode.window.setStatusBarMessage("OCL editor runtime missing bundled ocl-lsp", 5000);
    return;
  }

  const runtime = new OclLspRuntime(context, lspPath);
  await runtime.start();
  context.subscriptions.push(runtime);
  void vscode.window.setStatusBarMessage("OCL editor runtime ready (bundled LSP)", 4000);
}

function toLspPosition(position: vscode.Position): { line: number; character: number } {
  return {
    line: position.line,
    character: position.character,
  };
}

function toLspRange(range: vscode.Range): { start: { line: number; character: number }; end: { line: number; character: number } } {
  return {
    start: toLspPosition(range.start),
    end: toLspPosition(range.end),
  };
}

function fromLocation(value: any): vscode.Location | null {
  if (!value?.uri || !value?.range) {
    return null;
  }
  try {
    return new vscode.Location(vscode.Uri.parse(String(value.uri)), fromRange(value.range));
  } catch {
    return null;
  }
}

function fromRange(value: any): vscode.Range {
  return new vscode.Range(
    new vscode.Position(Number(value.start?.line ?? 0), Number(value.start?.character ?? 0)),
    new vscode.Position(Number(value.end?.line ?? 0), Number(value.end?.character ?? 0))
  );
}

function fromDiagnostic(value: any): vscode.Diagnostic | null {
  if (!value?.range || typeof value.message !== "string") {
    return null;
  }
  const diagnostic = new vscode.Diagnostic(
    fromRange(value.range),
    value.message,
    fromSeverity(value.severity)
  );
  diagnostic.code = value.code;
  diagnostic.source = value.source ?? "ocl-lsp";
  return diagnostic;
}

function fromSeverity(value: any): vscode.DiagnosticSeverity {
  switch (Number(value ?? 1)) {
    case 2:
      return vscode.DiagnosticSeverity.Warning;
    case 3:
      return vscode.DiagnosticSeverity.Information;
    case 4:
      return vscode.DiagnosticSeverity.Hint;
    default:
      return vscode.DiagnosticSeverity.Error;
  }
}

function fromCompletionKind(value: any): vscode.CompletionItemKind {
  switch (Number(value ?? 6)) {
    case 3:
      return vscode.CompletionItemKind.Function;
    case 7:
      return vscode.CompletionItemKind.Struct;
    case 14:
      return vscode.CompletionItemKind.Keyword;
    default:
      return vscode.CompletionItemKind.Variable;
  }
}

function fromSymbolInformation(value: any): vscode.SymbolInformation | null {
  if (!value?.name || !value?.location) {
    return null;
  }
  const location = fromLocation(value.location);
  if (!location) {
    return null;
  }
  return new vscode.SymbolInformation(
    String(value.name),
    fromSymbolKind(value.kind),
    "",
    location
  );
}

function fromSymbolKind(value: any): vscode.SymbolKind {
  switch (Number(value ?? 13)) {
    case 10:
      return vscode.SymbolKind.Enum;
    case 12:
      return vscode.SymbolKind.Function;
    case 23:
      return vscode.SymbolKind.Struct;
    default:
      return vscode.SymbolKind.Variable;
  }
}

function fromTextEdit(value: any): vscode.TextEdit | null {
  if (!value?.range || typeof value.newText !== "string") {
    return null;
  }
  return new vscode.TextEdit(fromRange(value.range), value.newText);
}

function fromWorkspaceEdit(value: any): vscode.WorkspaceEdit {
  const edit = new vscode.WorkspaceEdit();
  const changes = value?.changes ?? {};
  for (const [uriText, textEdits] of Object.entries(changes)) {
    if (!Array.isArray(textEdits)) {
      continue;
    }
    const uri = vscode.Uri.parse(uriText);
    const edits = textEdits.map(fromTextEdit).filter((item): item is vscode.TextEdit => item !== null);
    edit.set(uri, edits);
  }
  return edit;
}
