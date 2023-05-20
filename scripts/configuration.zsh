#!/usr/bin/env zsh
# shellcheck disable=SC1091,SC2296

set -euo pipefail

script_path="${(%):-%N}"
script_dir="$(cd "$(dirname "${script_path}")" && pwd -P)"
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
    ln -fs "${dotdir}/zsh/.terminal.zsh" "${HOME}/.zsh.terminal.zsh"    
    ln -fs "${dotdir}/zsh/.path.zsh" "${HOME}/.zsh.path.zsh"    
    ln -fs "${dotdir}/zsh/.alias.zsh" "${HOME}/.zsh.alias.zsh"    
    ln -fs "${dotdir}/zsh/.local.zsh" "${HOME}/.zsh.local.zsh"    
    ln -fs "${dotdir}/zsh/.zshrc" "${HOME}/.zshrc"
    source "${HOME}/.zshrc"

    # system tool
    if ! (type system > /dev/null 2>&1); then
        sudo ln -fs "${script_dir}/system.zsh" "/usr/local/bin/system"
    fi
}

link