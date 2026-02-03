import * as vscode from 'vscode';
import { getApiClient } from '../api/client';
import { MutationInfo } from '../api/types';

// Decoration types for mutations
const activeVariantDecoration = vscode.window.createTextEditorDecorationType({
  backgroundColor: new vscode.ThemeColor('diffEditor.insertedTextBackground'),
  isWholeLine: true,
  overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.addedForeground'),
  overviewRulerLane: vscode.OverviewRulerLane.Left,
});

const inactiveVariantDecoration = vscode.window.createTextEditorDecorationType({
  backgroundColor: new vscode.ThemeColor('diffEditor.removedTextBackground'),
  opacity: '0.6',
  isWholeLine: true,
  overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.deletedForeground'),
  overviewRulerLane: vscode.OverviewRulerLane.Left,
});

const mutationMarkerDecoration = vscode.window.createTextEditorDecorationType({
  color: new vscode.ThemeColor('editorLineNumber.foreground'),
  fontStyle: 'italic',
});

export class MutationDecorationManager {
  private disposables: vscode.Disposable[] = [];
  private cachedMutations: Map<string, MutationInfo[]> = new Map();

  constructor() {
    // Update decorations when active editor changes
    this.disposables.push(
      vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor) {
          this.updateDecorations(editor);
        }
      })
    );

    // Update decorations when document changes
    this.disposables.push(
      vscode.workspace.onDidChangeTextDocument((event) => {
        const editor = vscode.window.activeTextEditor;
        if (editor && event.document === editor.document) {
          // Clear cache on document change
          this.cachedMutations.delete(event.document.uri.fsPath);
          this.updateDecorations(editor);
        }
      })
    );

    // Initial decoration
    if (vscode.window.activeTextEditor) {
      this.updateDecorations(vscode.window.activeTextEditor);
    }
  }

  async updateDecorations(editor: vscode.TextEditor): Promise<void> {
    const document = editor.document;
    const filePath = document.uri.fsPath;

    // Only process supported file types
    const supportedExtensions = ['.rs', '.py', '.js', '.ts', '.v', '.ml', '.hs'];
    if (!supportedExtensions.some((ext) => filePath.endsWith(ext))) {
      editor.setDecorations(activeVariantDecoration, []);
      editor.setDecorations(inactiveVariantDecoration, []);
      editor.setDecorations(mutationMarkerDecoration, []);
      return;
    }

    let mutations: MutationInfo[];

    // Try to get from cache first
    if (this.cachedMutations.has(filePath)) {
      mutations = this.cachedMutations.get(filePath)!;
    } else {
      // Try to fetch from server, fall back to local parsing
      try {
        const client = getApiClient();
        const result = await client.getFileMutations(filePath);
        mutations = result.mutations;
      } catch {
        mutations = this.parseLocalMutations(document);
      }
      this.cachedMutations.set(filePath, mutations);
    }

    const activeRanges: vscode.DecorationOptions[] = [];
    const inactiveRanges: vscode.DecorationOptions[] = [];
    const markerRanges: vscode.DecorationOptions[] = [];

    for (const mutation of mutations) {
      const startLine = mutation.line - 1;
      const endLine = mutation.end_line - 1;

      if (startLine < 0 || startLine >= document.lineCount) {
        continue;
      }

      // Marker line decoration
      const markerRange = new vscode.Range(startLine, 0, startLine, document.lineAt(startLine).text.length);
      markerRanges.push({
        range: markerRange,
        hoverMessage: mutation.active
          ? `**Active variant**: ${mutation.name}`
          : `**Inactive variant**: ${mutation.name} - Click CodeLens to activate`,
      });

      // Content range decoration (lines after the marker)
      if (startLine + 1 < document.lineCount && endLine > startLine) {
        const contentStart = startLine + 1;
        const contentEnd = Math.min(endLine - 1, document.lineCount - 1);

        if (contentEnd >= contentStart) {
          const contentRange = new vscode.Range(
            contentStart,
            0,
            contentEnd,
            document.lineAt(contentEnd).text.length
          );

          if (mutation.active) {
            activeRanges.push({
              range: contentRange,
              hoverMessage: `**Active variant**: ${mutation.name}`,
            });
          } else {
            inactiveRanges.push({
              range: contentRange,
              hoverMessage: `**Inactive variant**: ${mutation.name}\n\nClick CodeLens above to activate this variant.`,
            });
          }
        }
      }
    }

    editor.setDecorations(activeVariantDecoration, activeRanges);
    editor.setDecorations(inactiveVariantDecoration, inactiveRanges);
    editor.setDecorations(mutationMarkerDecoration, markerRanges);
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

  refresh(): void {
    this.cachedMutations.clear();
    if (vscode.window.activeTextEditor) {
      this.updateDecorations(vscode.window.activeTextEditor);
    }
  }

  dispose(): void {
    activeVariantDecoration.dispose();
    inactiveVariantDecoration.dispose();
    mutationMarkerDecoration.dispose();
    this.disposables.forEach((d) => d.dispose());
  }
}

let manager: MutationDecorationManager | undefined;

export function getMutationDecorationManager(): MutationDecorationManager {
  if (!manager) {
    manager = new MutationDecorationManager();
  }
  return manager;
}

export function disposeMutationDecorationManager(): void {
  if (manager) {
    manager.dispose();
    manager = undefined;
  }
}
