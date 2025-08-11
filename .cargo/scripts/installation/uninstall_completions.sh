#!/bin/bash

USER_DIR="$HOME/.bash_completion.d"
GLOBAL_DIR="/etc/bash_completion.d"

uninstall_paths=(
    "$USER_DIR/jam_completion.sh"
    "$GLOBAL_DIR/jam_completion.sh"
)

found=false
for path in "${uninstall_paths[@]}"; do
    if [[ -f "$path" ]]; then
        found=true
        break
    fi
done

if ! $found; then
    echo "Err: Cannot found JamReady CLI completion script"
    exit 1
fi

read -p "Remove JamReady CLI Completion? [y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy] ]]; then
    echo "Canceled"
    exit 0
fi

removed_files=0
removed_from_bashrc=false
BASHRC="$HOME/.bashrc"

for path in "${uninstall_paths[@]}"; do
    if [[ -f "$path" ]]; then
        rm -f "$path"
        echo "Removed: $path"
        ((removed_files++))
    fi
done

if [[ -f "$BASHRC" ]]; then
    if grep -q "jam_completion.sh" "$BASHRC"; then
        grep -v "jam_completion.sh" "$BASHRC" > "$BASHRC.tmp"
        mv "$BASHRC.tmp" "$BASHRC"
        echo "Removed configure from $BASHRC"
        removed_from_bashrc=true
    fi
fi

if [[ $removed_files -eq 0 ]]; then
    echo "No file to remove"
else
    echo -e "\nUninstalled JamReady CLI Completion"
    echo "Removed $removed_files file(s)."
fi