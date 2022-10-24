#!/bin/bash -ue

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

  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
  local dotdir
  dotdir=$(dirname "${script_dir}")

  command echo "Start to link dotfiles to the home directory."
  if [[ "${HOME}" != "${dotdir}" ]];then
    for f in "${dotdir}"/.??*; do
      [[ $(basename "$f") == ".git" ]] && continue
      if [[ -L "${HOME}/$(basename "$f")" ]];then
        # command rm -f "${HOME}/$(basename "$f")"
        echo "command rm -f ${HOME}/$(basename "$f")"
      fi
      if [[ -e "${HOME}/$(basename "$f")" ]];then
        # command mv "${HOME}/$(basename "$f")" "${HOME}/.dotbackup"
        echo "command mv ${HOME}/$(basename "$f") ${HOME}/.dotbackup"
      fi
      #command ln -snf $f ${HOME}
      echo "command ln -snf $f ${HOME}"
    done
  else
    command echo "Home and dotfiles are same directory. Please change directory."
  fi
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