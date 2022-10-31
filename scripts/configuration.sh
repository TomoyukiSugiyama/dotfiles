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
        exit 1
    fi

    # git
    ln -fs "${dotdir}/git/.gitignore" "${HOME}/.gitignore"
    ln -fs "${dotdir}/git/.gitconfig" "${HOME}/.gitconfig"
    ln -fs /usr/local/opt/git/share/git-core/contrib/diff-highlight/diff-highlight  /usr/local/bin
    if [[ ! -e "${HOME}/.gitconfig.local" ]]; then
        echo "copy .gitconfig.local"
        cp "${dotdir}/git/.gitconfig.local" "${HOME}/.gitconfig.local"
    fi

}

link