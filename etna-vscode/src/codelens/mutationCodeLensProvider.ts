import * as vscode from 'vscode';
import { getApiClient } from '../api/client';
import { MutationInfo } from '../api/types';

export class MutationCodeLensProvider implements vscode.CodeLensProvider {
  private _onDidChangeCodeLenses: vscode.EventEmitter<void> = new vscode.EventEmitter<void>();
  public readonly onDidChangeCodeLenses: vscode.Event<void> = this._onDidChangeCodeLenses.event;

  private cachedMutations: Map<string, MutationInfo[]> = new Map();
  private lastRefreshTime: Map<string, number> = new Map();
  private readonly CACHE_TTL_MS = 2000; // 2 second cache

  constructor() {
    // Refresh on file save
    vscode.workspace.onDidSaveTextDocument(() => {
      this.refresh();
    });
  }

  public refresh(): void {
    this.cachedMutations.clear();
    this.lastRefreshTime.clear();
    this._onDidChangeCodeLenses.fire();
  }

  async provideCodeLenses(
    document: vscode.TextDocument,
    _token: vscode.CancellationToken
  ): Promise<vscode.CodeLens[]> {
    const filePath = document.uri.fsPath;

    // Check if we have a recent cache
    const lastRefresh = this.lastRefreshTime.get(filePath) || 0;
    const now = Date.now();

    let mutations: MutationInfo[];

    if (now - lastRefresh < this.CACHE_TTL_MS && this.cachedMutations.has(filePath)) {
      mutations = this.cachedMutations.get(filePath)!;
    } else {
      // Fetch from server
      try {
        const client = getApiClient();
        const result = await client.getFileMutations(filePath);
        mutations = result.mutations;
        this.cachedMutations.set(filePath, mutations);
        this.lastRefreshTime.set(filePath, now);
      } catch {
        // If server is not available, try to parse locally
        mutations = this.parseLocalMutations(document);
        this.cachedMutations.set(filePath, mutations);
        this.lastRefreshTime.set(filePath, now);
      }
    }

    const codeLenses: vscode.CodeLens[] = [];

    // Group mutations by their starting line to identify variation blocks
    const mutationsByLine = new Map<number, MutationInfo[]>();
    for (const mutation of mutations) {
      const lineArray = mutationsByLine.get(mutation.line) || [];
      lineArray.push(mutation);
      mutationsByLine.set(mutation.line, lineArray);
    }

    // Create CodeLens for each mutation
    for (const mutation of mutations) {
      const line = mutation.line - 1; // Convert to 0-indexed
      if (line < 0 || line >= document.lineCount) {
        continue;
      }

      const range = new vscode.Range(line, 0, line, document.lineAt(line).text.length);

      if (mutation.active) {
        // Active mutation - show check icon and option to deactivate
        codeLenses.push(
          new vscode.CodeLens(range, {
            title: `$(check) ${mutation.name} (active)`,
            tooltip: 'This mutation variant is currently active. Click to see options.',
            command: 'etna.showMutationOptions',
            arguments: [mutation, filePath],
          })
        );
      } else {
        // Inactive mutation - show circle and option to activate
        codeLenses.push(
          new vscode.CodeLens(range, {
            title: `$(circle-outline) ${mutation.name}`,
            tooltip: `Click to activate mutation variant "${mutation.name}"`,
            command: 'etna.activateMutation',
            arguments: [mutation, filePath],
          })
        );
      }
    }

    // Add reset button at the top if there are mutations
    if (mutations.length > 0) {
      const firstLine = Math.min(...mutations.map((m) => m.line)) - 1;
      if (firstLine >= 0) {
        const range = new vscode.Range(firstLine, 0, firstLine, 0);
        codeLenses.push(
          new vscode.CodeLens(range, {
            title: '$(debug-restart) Reset Mutations',
            tooltip: 'Reset all mutations to their default state',
            command: 'etna.resetMutations',
            arguments: [filePath],
          })
        );
      }
    }

    return codeLenses;
  }

  private parseLocalMutations(document: vscode.TextDocument): MutationInfo[] {
    const mutations: MutationInfo[] = [];
    const text = document.getText();
    const lines = text.split('\n');

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      const lineNum = i + 1;

      // Check for active variant marker: /*| name */
      const activeMatch = line.match(/^\/\*\|\s+(\w+)\s+\*\/$/);
      if (activeMatch && !line.startsWith('/*||')) {
        mutations.push({
          name: activeMatch[1],
          active: true,
          file: document.uri.fsPath,
          line: lineNum,
          end_line: this.findVariationEnd(lines, i),
        });
      }

      // Check for inactive variant marker: /*|| name */
      const inactiveMatch = line.match(/^\/\*\|\|\s+(\w+)\s+\*\/$/);
      if (inactiveMatch) {
        mutations.push({
          name: inactiveMatch[1],
          active: false,
          file: document.uri.fsPath,
          line: lineNum,
          end_line: this.findVariantEnd(lines, i),
        });
      }
    }

    return mutations;
  }

  private findVariationEnd(lines: string[], start: number): number {
    for (let i = start + 1; i < lines.length; i++) {
      if (lines[i].trim() === '/* |*/') {
        return i + 1;
      }
    }
    return lines.length;
  }

  private findVariantEnd(lines: string[], start: number): number {
    for (let i = start + 1; i < lines.length; i++) {
      const trimmed = lines[i].trim();
      if (trimmed.startsWith('/*||') || trimmed === '/* |*/') {
        return i;
      }
    }
    return lines.length;
  }
}

let provider: MutationCodeLensProvider | undefined;

export function getMutationCodeLensProvider(): MutationCodeLensProvider {
  if (!provider) {
    provider = new MutationCodeLensProvider();
  }
  return provider;
}
