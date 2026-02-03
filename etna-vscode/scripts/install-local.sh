#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

echo "==> Installing dependencies..."
if [ ! -d node_modules ]; then
  npm install
fi
if [ ! -d webview-ui/node_modules ]; then
  (cd webview-ui && npm install)
fi

echo "==> Building webview UI..."
(cd webview-ui && npm run build)

echo "==> Packaging extension (.vsix)..."
npx @vscode/vsce package --no-dependencies --allow-missing-repository 

echo "==> Installing extension into VS Code..."
code --install-extension etna-vscode-*.vsix

echo "==> Done! Restart VS Code and run 'Etna: Open Dashboard' from the command palette."
