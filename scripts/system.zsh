#!/usr/bin/env zsh
# shellcheck disable=SC2296

set -euo pipefail

linked_script_path="${(%):-%N}"
script_path="$(readlink "${linked_script_path}")"
script_dir="$(cd "$(dirname "${script_path}")" && pwd -P)"

function help() {
    echo "Usage:"
    echo "    ${script_path} [--help | -h] [--update | -u]" 0>&2
    echo "Options:"
    echo "    --help, -h        help message"
    echo "    --update, -u      system update"
}

function update() {
    cd "${script_dir}"
    git remote update --prune
    git checkout main
    git branch --set-upstream-to="origin/main" "main"
    git pull

    "${script_dir}"/bootstrap.zsh
    "${script_dir}"/configuration.zsh
}

while [ $# -gt 0 ];do
    case ${1} in
        --help|-h)
            help
            exit 1
            ;;
        --update|-u)
            update
            exit 1
            ;;
        *)
            ;;
    esac
    shift
done

update