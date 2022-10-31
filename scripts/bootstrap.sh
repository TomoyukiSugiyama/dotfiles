#!/bin/bash -ue

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

function help() {
    command echo "Usage:"
    command echo "    $(basename "${0}") [--help | -h]" 0>&2
    command echo "Options:"
    command echo "    --help, -h        help message"
}

function setup_brew() {
    command echo "setup brew"
    if ! (type brew > /dev/null 2>&1); then
        command echo "Homebrew is not found in your local pc. Begin to install homebrew."
        command bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        echo '# Set PATH, MANPATH, etc., for Homebrew.' >> ~/.zprofile
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
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