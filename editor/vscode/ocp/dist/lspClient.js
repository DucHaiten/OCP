"use strict";

const { spawn } = require("child_process");
const vscode = require("vscode");
const { resolveRuntimePath } = require("./runtimePaths");

const LANGUAGE_ID = "ocp";
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

class OcpLspRuntime {
  constructor(context, binaryPath) {
    this.context = context;
    this.binaryPath = binaryPath;
    this.diagnostics = vscode.languages.createDiagnosticCollection("ocp");
    this.docSelector = [{ language: LANGUAGE_ID, scheme: "file" }];
    this.registrations = [];
    this.pending = new Map();
    this.openedUris = new Set();
    this.semanticLegend = new vscode.SemanticTokensLegend(SEMANTIC_TOKEN_TYPES, []);
    this.process = null;
    this.nextId = 1;
    this.readBuffer = Buffer.alloc(0);
    this.disposed = false;
  }

  async start() {
    this.process = spawn(this.binaryPath, [], {
      cwd: this.workspaceRoot(),
      stdio: "pipe",
    });

    this.process.stdout.on("data", (chunk) => {
      this.readBuffer = Buffer.concat([this.readBuffer, chunk]);
      this.drainMessages();
    });
    this.process.stderr.on("data", () => {});
    this.process.on("exit", () => {
      if (!this.disposed) {
        void vscode.window.showErrorMessage("OCP language runtime stopped unexpectedly.");
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

  dispose() {
    this.disposed = true;
    for (const pending of this.pending.values()) {
      pending.reject(new Error("OCP language runtime disposed."));
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

  workspaceRoot() {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  }

  registerProviders() {
    this.registrations.push(this.diagnostics);
    this.registrations.push(
      vscode.workspace.onDidOpenTextDocument(async (document) => {
        await this.syncDocument(document, "open");
      }),
    );
    this.registrations.push(
      vscode.workspace.onDidChangeTextDocument(async (event) => {
        await this.syncDocument(event.document, "change");
      }),
    );
    this.registrations.push(
      vscode.workspace.onDidSaveTextDocument(async (document) => {
        await this.syncDocument(document, "save");
      }),
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
      }),
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
      }),
    );
    this.registrations.push(
      vscode.languages.registerCompletionItemProvider(this.docSelector, {
        provideCompletionItems: async (document, position) => {
          const result = await this.request("textDocument/completion", {
            textDocument: { uri: document.uri.toString() },
            position: toLspPosition(position),
          });
          const items = Array.isArray(result?.items) ? result.items : [];
          return items.map((item) => {
            const completion = new vscode.CompletionItem(
              String(item.label ?? ""),
              fromCompletionKind(item.kind),
            );
            return completion;
          });
        },
      }),
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
          return result.map(fromLocation).filter((item) => item !== null);
        },
      }),
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
          return result.map(fromLocation).filter((item) => item !== null);
        },
      }),
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
      }),
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
          return result.map(fromSymbolInformation).filter((item) => item !== null);
        },
      }),
    );
    this.registrations.push(
      vscode.languages.registerWorkspaceSymbolProvider({
        provideWorkspaceSymbols: async (query) => {
          const result = await this.request("workspace/symbol", { query });
          if (!Array.isArray(result)) {
            return [];
          }
          return result.map(fromSymbolInformation).filter((item) => item !== null);
        },
      }),
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
          return result.map(fromTextEdit).filter((item) => item !== null);
        },
      }),
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
          return result.map((item) => {
            const action = new vscode.CodeAction(
              String(item.title ?? "OCP Code Action"),
              vscode.CodeActionKind.QuickFix,
            );
            if (item?.command?.command) {
              action.command = {
                command: String(item.command.command),
                title: String(item.command.title ?? item.title ?? "OCP Code Action"),
              };
            }
            return action;
          });
        },
      }),
    );

    if (vscode.workspace.getConfiguration("ocp").get("enableSemanticTokens", true)) {
      this.registrations.push(
        vscode.languages.registerDocumentSemanticTokensProvider(
          this.docSelector,
          {
            provideDocumentSemanticTokens: async (document) => {
              const result = await this.request("textDocument/semanticTokens/full", {
                textDocument: { uri: document.uri.toString() },
              });
              const data = Array.isArray(result?.data)
                ? result.data.map((value) => Number(value))
                : [];
              return new vscode.SemanticTokens(new Uint32Array(data));
            },
          },
          this.semanticLegend,
        ),
      );
    }
  }

  async syncOpenDocuments() {
    const docs = vscode.workspace.textDocuments.filter(
      (document) => document.languageId === LANGUAGE_ID && document.uri.scheme === "file",
    );
    for (const document of docs) {
      await this.syncDocument(document, "open");
    }
  }

  async syncDocument(document, mode) {
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

  request(method, params) {
    if (!this.process) {
      return Promise.reject(new Error("OCP language runtime is not running."));
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

  notify(method, params) {
    if (!this.process) {
      return;
    }
    this.writeMessage({
      jsonrpc: "2.0",
      method,
      params,
    });
  }

  writeMessage(payload) {
    if (!this.process) {
      return;
    }
    const body = Buffer.from(JSON.stringify(payload), "utf8");
    this.process.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
    this.process.stdin.write(body);
  }

  drainMessages() {
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

  onMessage(message) {
    if (typeof message?.id === "number" && this.pending.has(message.id)) {
      const pending = this.pending.get(message.id);
      this.pending.delete(message.id);
      if (!pending) {
        return;
      }
      if (message.error) {
        pending.reject(new Error(String(message.error.message ?? "OCP LSP request failed.")));
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
        ? message.params.diagnostics.map(fromDiagnostic).filter((item) => item !== null)
        : [];
      this.diagnostics.set(uri, diagnostics);
    }
  }
}

async function startLanguageClient(context) {
  if (!vscode.workspace.isTrusted) {
    await vscode.commands.executeCommand("setContext", "ocp.workspaceTrusted", false);
    void vscode.window.setStatusBarMessage("OCP editor runtime disabled: workspace is untrusted", 4000);
    return;
  }

  await vscode.commands.executeCommand("setContext", "ocp.workspaceTrusted", true);
  const lspPath = resolveRuntimePath(context, "ocp-lsp.exe");
  if (!lspPath) {
    void vscode.window.setStatusBarMessage("OCP editor runtime missing bundled ocp-lsp", 5000);
    return;
  }

  const runtime = new OcpLspRuntime(context, lspPath);
  await runtime.start();
  context.subscriptions.push(runtime);
  void vscode.window.setStatusBarMessage("OCP editor runtime ready (bundled LSP)", 4000);
}

function toLspPosition(position) {
  return {
    line: position.line,
    character: position.character,
  };
}

function toLspRange(range) {
  return {
    start: toLspPosition(range.start),
    end: toLspPosition(range.end),
  };
}

function fromLocation(value) {
  if (!value?.uri || !value?.range) {
    return null;
  }
  try {
    return new vscode.Location(vscode.Uri.parse(String(value.uri)), fromRange(value.range));
  } catch {
    return null;
  }
}

function fromRange(value) {
  return new vscode.Range(
    new vscode.Position(Number(value.start?.line ?? 0), Number(value.start?.character ?? 0)),
    new vscode.Position(Number(value.end?.line ?? 0), Number(value.end?.character ?? 0)),
  );
}

function fromDiagnostic(value) {
  if (!value?.range || typeof value.message !== "string") {
    return null;
  }
  const diagnostic = new vscode.Diagnostic(
    fromRange(value.range),
    value.message,
    fromSeverity(value.severity),
  );
  diagnostic.code = value.code;
  diagnostic.source = value.source ?? "ocp-lsp";
  return diagnostic;
}

function fromSeverity(value) {
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

function fromCompletionKind(value) {
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

function fromSymbolInformation(value) {
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
    location,
  );
}

function fromSymbolKind(value) {
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

function fromTextEdit(value) {
  if (!value?.range || typeof value.newText !== "string") {
    return null;
  }
  return new vscode.TextEdit(fromRange(value.range), value.newText);
}

function fromWorkspaceEdit(value) {
  const edit = new vscode.WorkspaceEdit();
  const changes = value?.changes ?? {};
  for (const [uriText, textEdits] of Object.entries(changes)) {
    if (!Array.isArray(textEdits)) {
      continue;
    }
    const uri = vscode.Uri.parse(uriText);
    const edits = textEdits.map(fromTextEdit).filter((item) => item !== null);
    edit.set(uri, edits);
  }
  return edit;
}

module.exports = {
  startLanguageClient,
};
