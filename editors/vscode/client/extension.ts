import { promises as fsPromises } from "node:fs";
import { join } from "node:path";
import {
	commands,
	ExtensionContext,
	StatusBarAlignment,
	StatusBarItem,
	ThemeColor,
	window,
	workspace,
} from "vscode";
import { MessageType, ShowMessageNotification } from "vscode-languageclient";
import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
} from "vscode-languageclient/node";

import { ConfigService } from "./config";

const languageClientName = "oxc";

const outputChannelName = "Oxc";

import { join } from 'node:path';
import { ConfigService } from './ConfigService';

const languageClientName = 'oxc';
const outputChannelName = 'Oxc';
const commandPrefix = 'oxc';

const enum OxcCommands {
	RestartServer = `${commandPrefix}.restartServer`,
	ApplyAllFixes = `${commandPrefix}.applyAllFixes`,
	ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
	ToggleEnable = `${commandPrefix}.toggleEnable`,
}

let client: LanguageClient;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();
  const restartCommand = commands.registerCommand(
    OxcCommands.RestartServer,
    async () => {
      if (!client) {
        window.showErrorMessage('oxc client not found');
        return;
      }

				return;
			}

			try {
				if (client.isRunning()) {
					await client.restart();

					window.showInformationMessage("oxc server restarted.");
				} else {
					await client.start();
				}
			} catch (err) {
				client.error("Restarting client failed", err, "force");
			}
		},
	);

  const toggleEnable = commands.registerCommand(
    OxcCommands.ToggleEnable,
    () => {
      configService.config.updateEnable(!configService.config.enable);
    },
  );

  context.subscriptions.push(
    restartCommand,
    showOutputCommand,
    toggleEnable,
    configService,
  );

	context.subscriptions.push(
		restartCommand,
		showOutputCommand,
		toggleEnable,
		config,
	);

  async function findBinary(): Promise<string> {
    let bin = configService.config.binPath;
    if (bin) {
      try {
        await fsPromises.access(bin);
        return bin;
      } catch {}
    }

	async function findBinary(): Promise<string> {
		let bin = config.binPath;

		if (bin) {
			try {
				await fsPromises.access(bin);

				return bin;
			} catch {}
		}

		const workspaceFolders = workspace.workspaceFolders;

  const command = await findBinary();
  const run: Executable = {
    command: command!,
    options: {
      env: {
        ...process.env,
        RUST_LOG: process.env.RUST_LOG || 'info',
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      'typescript',
      'javascript',
      'typescriptreact',
      'javascriptreact',
      'vue',
      'svelte',
    ].map((lang) => ({
      language: lang,
      scheme: 'file',
    })),
    synchronize: {
      // Notify the server about file config changes in the workspace
      fileEvents: [
        workspace.createFileSystemWatcher('**/.eslintr{c,c.json}'),
        workspace.createFileSystemWatcher('**/.oxlint{.json,rc.json,rc}'),
        workspace.createFileSystemWatcher('**/oxlint{.json,rc.json}'),
      ],
    },
    initializationOptions: {
      settings: configService.config.toLanguageServerConfig(),
    },
    outputChannel,
    traceOutputChannel: outputChannel,
  };

		if (workspaceFolders?.length && !isWindows) {
			try {
				return await Promise.any(
					workspaceFolders.map(async (folder) => {
						const binPath = join(
							folder.uri.fsPath,
							"node_modules",
							".bin",
							"oxc_language_server",
						);

						await fsPromises.access(binPath);

  configService.onConfigChange = function onConfigChange() {
    let settings = this.config.toLanguageServerConfig();
    updateStatsBar(settings.enable);
    client.sendNotification('workspace/didChangeConfiguration', { settings });
  };

		const ext = isWindows ? ".exe" : "";
		// NOTE: The `./target/release` path is aligned with the path defined in .github/workflows/release_vscode.yml
		return (
			process.env.SERVER_PATH_DEV ??
			join(
				context.extensionPath,
				`./target/release/oxc_language_server${ext}`,
			)
		);
	}

    myStatusBarItem.backgroundColor = bgColor;
  }
  updateStatsBar(configService.config.enable);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
