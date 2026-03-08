"use strict";

const { registerCommands } = require("./commands");
const { startDebugRuntime } = require("./dapRuntime");
const { startLanguageClient } = require("./lspClient");

async function activate(context) {
  registerCommands(context);
  await startLanguageClient(context);
  await startDebugRuntime(context);
}

async function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
