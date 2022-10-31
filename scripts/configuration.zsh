#!/usr/bin/env zsh

set -euo pipefail

script_dir="$(cd "$(dirname "${(%):-%N}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function link() {

    echo "Create backup directory for old dotfiles..."

    if [ ! -d "${HOME}/.dotbackup" ];then
        echo "${HOME}/.dotbackup not found. Generate .dotbackup directory."
        mkdir "${HOME}/.dotbackup"
    fi

    echo "Start to link dotfiles to the home directory."
    if [[ "${HOME}" == "${dotdir}" ]];then
        echo "[Error] Home directory and dotfiles directory are same path. Please change your home or dotfiles directory path."
        exit 1
    fi

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