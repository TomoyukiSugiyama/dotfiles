#!/usr/bin/env zsh
# shellcheck disable=SC2296 # https://www.shellcheck.net/wiki/SC2296

set -euo pipefail

script_path="${(%):-%N}"
script_dir="$(cd "$(dirname "${script_path}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function help() {
    echo "Usage:"
    echo "    ${script_path} [--help | -h]" 0>&2
    echo "Options:"
    echo "    --help, -h        help message"
}

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