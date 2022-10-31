#!/usr/bin/env zsh

set -euo pipefail

script_dir="$(cd "$(dirname "${(%):-%N}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function help() {
    echo "Usage:"
    echo "    $(basename "${0}") [--help | -h]" 0>&2
    echo "Options:"
    echo "    --help, -h        help message"
}

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

while [ $# -gt 0 ];do
    case ${1} in
        --help|-h)
            help
            exit 1
            ;;
        *)
            ;;
    esac
    shift
done

link