#!/usr/bin/env zsh

set -euo pipefail

script_dir="$(cd "$(dirname "${(%):-%N}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function link() {
    echo "Start to link dotfiles to the home directory."

    # git
    ln -fs "${dotdir}/git/.gitignore" "${HOME}/.gitignore"
    ln -fs "${dotdir}/git/.gitconfig" "${HOME}/.gitconfig"
    if [[ ! -e "${HOME}/.gitconfig.local" ]]; then
        echo "copy .gitconfig.local"
        cp "${dotdir}/git/.gitconfig.local" "${HOME}/.gitconfig.local"
    fi

    # zsh
    ln -fs "${dotdir}/zsh/.zshrc" "${HOME}/.zshrc"
    source "${HOME}/.zshrc"
}

link