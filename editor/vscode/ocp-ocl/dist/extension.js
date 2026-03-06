"use strict";

const { registerCommands } = require("./commands");
const { startLanguageClient } = require("./lspClient");

async function activate(context) {
  registerCommands(context);
  await startLanguageClient(context);
}

async function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
