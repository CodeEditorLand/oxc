import { ExtensionContext, StatusBarAlignment, StatusBarItem, ThemeColor, window, workspace } from 'vscode';

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  MessageType,
  ServerOptions,
  ShowMessageNotification,
} from 'vscode-languageclient/node';

import {
  applyAllFixesFileCommand,
  OxcCommands,
  restartServerCommand,
  showOutputChannelCommand,
  toggleEnabledCommand,
} from './commands';
import { ConfigService } from './ConfigService';
import findBinary from './findBinary';

const languageClientName = 'oxc';
const outputChannelName = 'Oxc';

let client: LanguageClient;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();

  context.subscriptions.push(
    applyAllFixesFileCommand(client),
    restartServerCommand(client),
    showOutputChannelCommand(client),
    toggleEnabledCommand(configService.config),
    configService,
  );

	context.subscriptions.push(
		restartCommand,
		showOutputCommand,
		toggleEnable,
		config,
	);

  const command = await findBinary(context, configService.config);
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
        workspace.createFileSystemWatcher('**/.oxlint{.json,rc.json}'),
        workspace.createFileSystemWatcher('**/oxlint{.json,rc.json}'),
      ],
    },
    initializationOptions: {
      settings: configService.config.toLanguageServerConfig(),
    },
    outputChannel,
    traceOutputChannel: outputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    languageClientName,
    serverOptions,
    clientOptions,
  );

  client.onNotification(ShowMessageNotification.type, (params) => {
    switch (params.type) {
      case MessageType.Debug:
        outputChannel.debug(params.message);
        break;
      case MessageType.Log:
        outputChannel.info(params.message);
        break;
      case MessageType.Info:
        window.showInformationMessage(params.message);
        break;
      case MessageType.Warning:
        window.showWarningMessage(params.message);
        break;
      case MessageType.Error:
        window.showErrorMessage(params.message);
        break;
      default:
        outputChannel.info(params.message);
    }
  });

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
