import * as vscode from 'vscode';
import {
  activateMutation,
  deactivateMutation,
  showMutationOptions,
  resetMutations,
  refreshMutations,
} from './mutationCommands';

export function registerCommands(context: vscode.ExtensionContext): void {
  // Mutation commands
  context.subscriptions.push(
    vscode.commands.registerCommand('etna.activateMutation', activateMutation)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('etna.deactivateMutation', deactivateMutation)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('etna.showMutationOptions', showMutationOptions)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('etna.resetMutations', resetMutations)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('etna.refreshMutations', refreshMutations)
  );
}
