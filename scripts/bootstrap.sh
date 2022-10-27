#!/bin/bash -ue

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function help() {
    command echo "Usage:"
    command echo "    $(basename "${0}") [--help | -h]" 0>&2
    command echo "Options:"
    command echo "    --help, -h        help message"
}

function link() {
    command echo "Create backup directory for old dotfiles..."
    if [ ! -d "${HOME}/.dotbackup" ];then
        command echo "${HOME}/.dotbackup not found. Generate .dotbackup directory."
        command mkdir "${HOME}/.dotbackup"
    fi

    command echo "Start to link dotfiles to the home directory."
    if [[ "${HOME}" != "${dotdir}" ]];then
        for f in "${dotdir}"/.??*; do
            [[ $(basename "$f") == ".git" ]] && continue
            if [[ -L "${HOME}/$(basename "$f")" ]];then
                # command rm -f "${HOME}/$(basename "$f")"
                command echo "command rm -f ${HOME}/$(basename "$f")"
            fi
            if [[ -e "${HOME}/$(basename "$f")" ]];then
                # command mv "${HOME}/$(basename "$f")" "${HOME}/.dotbackup"
                command echo "command mv ${HOME}/$(basename "$f") ${HOME}/.dotbackup"
            fi
            #command ln -snf $f ${HOME}
            command echo "command ln -snf $f ${HOME}"
        done
    else
        command echo "[Error] Home directory and dotfiles directory are same path. Please change your home or dotfiles directory path."
    fi
}

function setup_brew() {
    command echo "setup brew"
    if ! (type brew > /dev/null 2>&1); then
        command echo "Homebrew is not found in your local pc. Begin to install homebrew."
        command curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh
    fi
    command brew analytics off
    command brew cleanup --prune=all
    command brew upgrade
    command brew bundle --file "${dotdir}/Brewfile"
    command brew doctor || true
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

setup_brew
link