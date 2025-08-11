#!/bin/bash

USER_DIR="$HOME/.bash_completion.d"
GLOBAL_DIR="/etc/bash_completion.d"

if [[ $EUID -eq 0 ]]; then
    INSTALL_DIR="$GLOBAL_DIR"
    INSTALL_TYPE="All users"
else
    INSTALL_DIR="$USER_DIR"
    INSTALL_TYPE="Current users"
fi

mkdir -p "$INSTALL_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPLETION_SCRIPT="$SCRIPT_DIR/completion_script_jam.sh"

if [[ ! -f "$COMPLETION_SCRIPT" ]]; then
    echo "Err: Cannot found completion_script_jam.sh"
    exit 1
fi

INSTALL_PATH="$INSTALL_DIR/jam_completion.sh"
cp "$COMPLETION_SCRIPT" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

BASHRC_FILE="$HOME/.bashrc"
if ! grep -q "source.*jam_completion.sh" "$BASHRC_FILE" 2>/dev/null; then
    echo -e "\n# JamReady CLI Completion" >> "$BASHRC_FILE"
    echo "[[ -f \"$INSTALL_PATH\" ]] && source \"$INSTALL_PATH\"" >> "$BASHRC_FILE"
    ADDED_TO_BASHRC=true
fi

echo -e "\nInstalled ($INSTALL_TYPE)"

source "$INSTALL_PATH" 2>/dev/null