#!/bin/bash -ue

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function link() {
    command echo "Create backup directory for old dotfiles..."
    if [ ! -d "${HOME}/.dotbackup" ];then
        command echo "${HOME}/.dotbackup not found. Generate .dotbackup directory."
        command mkdir "${HOME}/.dotbackup"
    fi

    command echo "Start to link dotfiles to the home directory."
    if [[ "${HOME}" == "${dotdir}" ]];then
        command echo "[Error] Home directory and dotfiles directory are same path. Please change your home or dotfiles directory path."
        command exit 1
    fi

    # git
    command ln -fs "${dotdir}/git/.gitignore" "${HOME}/.gitignore"
    command ln -fs "${dotdir}/git/.gitconfig" "${HOME}/.gitconfig"
    if [[ ! -e "${HOME}/.gitconfig.local" ]]; then
        command echo "copy .gitconfig.local"
        command cp "${dotdir}/git/.gitconfig.local" "${HOME}/.gitconfig.local"
    fi

    # zsh
    command ln -fs "${dotdir}/zsh/.zshrc" "${HOME}/.zshrc"
    command zsh -c "$(source ${HOME}/.zshrc)"
}

link