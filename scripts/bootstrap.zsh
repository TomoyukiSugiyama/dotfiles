#!/usr/bin/env zsh
# shellcheck disable=SC2296

set -euo pipefail

script_path="${(%):-%N}"
script_dir="$(cd "$(dirname "${script_path}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function setup_brew() {
    echo "setup brew"
    if ! (type brew > /dev/null 2>&1); then
        echo "Homebrew is not found in your local pc. Begin to install homebrew."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        echo "# Set PATH, MANPATH, etc., for Homebrew." >> ~/.zprofile
        echo "eval \"\$(/opt/homebrew/bin/brew shellenv)\"" >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
    brew analytics off
    brew cleanup --prune=all
    brew upgrade
    brew bundle --file "${dotdir}/Brewfile"
    brew doctor || true
}

setup_brew