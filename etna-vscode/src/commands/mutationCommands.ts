import * as vscode from 'vscode';
import * as path from 'path';
import { getApiClient } from '../api/client';
import { MutationInfo } from '../api/types';
import { getMutationCodeLensProvider } from '../codelens/mutationCodeLensProvider';
import { getMutationDecorationManager } from '../codelens/mutationDecorations';

export async function activateMutation(mutation: MutationInfo, filePath: string): Promise<void> {
  const client = getApiClient();

  try {
    // Get the directory containing the file
    const fileDir = path.dirname(filePath);

    await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: `Activating mutation: ${mutation.name}`,
        cancellable: false,
      },
      async () => {
        await client.setMutation({
          path: fileDir,
          variant: mutation.name,
        });
      }
    );

    vscode.window.showInformationMessage(`Activated mutation variant: ${mutation.name}`);

    // Refresh CodeLens and decorations
    getMutationCodeLensProvider().refresh();
    getMutationDecorationManager().refresh();
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to activate mutation: ${message}`);
  }
}

export async function deactivateMutation(mutation: MutationInfo, filePath: string): Promise<void> {
  // Deactivating is the same as resetting for the file's directory
  await resetMutationsInPath(path.dirname(filePath));
}

export async function showMutationOptions(mutation: MutationInfo, filePath: string): Promise<void> {
  const options = [
    {
      label: '$(debug-restart) Reset to Default',
      description: 'Reset all mutations in this file to their default state',
      action: 'reset',
    },
    {
      label: '$(info) View Mutation Info',
      description: `Variant: ${mutation.name}, Lines: ${mutation.line}-${mutation.end_line}`,
      action: 'info',
    },
  ];

  const selected = await vscode.window.showQuickPick(options, {
    placeHolder: `Mutation: ${mutation.name} (active)`,
  });

  if (!selected) {
    return;
  }

  switch (selected.action) {
    case 'reset':
      await resetMutationsInPath(path.dirname(filePath));
      break;
    case 'info':
      vscode.window.showInformationMessage(
        `Mutation "${mutation.name}" is active at lines ${mutation.line}-${mutation.end_line} in ${path.basename(filePath)}`
      );
      break;
  }
}

export async function resetMutations(filePath?: string): Promise<void> {
  let targetPath: string;

  if (filePath) {
    targetPath = path.dirname(filePath);
  } else {
    // If no file path provided, try to get from active editor
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No file open to reset mutations for');
      return;
    }
    targetPath = path.dirname(editor.document.uri.fsPath);
  }

  await resetMutationsInPath(targetPath);
}

async function resetMutationsInPath(targetPath: string): Promise<void> {
  const client = getApiClient();

  try {
    await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Resetting mutations...',
        cancellable: false,
      },
      async () => {
        await client.resetMutations({ path: targetPath });
      }
    );

    vscode.window.showInformationMessage('Mutations reset to default state');

    // Refresh CodeLens and decorations
    getMutationCodeLensProvider().refresh();
    getMutationDecorationManager().refresh();
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    vscode.window.showErrorMessage(`Failed to reset mutations: ${message}`);
  }
}

export function refreshMutations(): void {
  getMutationCodeLensProvider().refresh();
  getMutationDecorationManager().refresh();
  vscode.window.showInformationMessage('Mutations refreshed');
}
