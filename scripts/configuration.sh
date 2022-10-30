#!/bin/bash -ue

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

link