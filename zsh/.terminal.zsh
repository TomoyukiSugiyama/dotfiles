#!/usr/bin/env zsh
# shellcheck disable=SC2034,SC2154

export CLICOLOR=1
export TERM=xterm-256color

autoload -Uz colors
colors

autoload -Uz compinit
compinit

PROMPT="%{${fg[blue]}%}%n:%{${reset_color}%} %c/ %# "

setopt no_beep

# vcs_info
autoload -Uz vcs_info
autoload -Uz add-zsh-hook
 
zstyle ':vcs_info:*' formats '%F{green}(%s)-[%b]%f'
zstyle ':vcs_info:*' actionformats '%F{red}(%s)-[%b|%a]%f'
 
function _update_vcs_info_msg() {
    LANG=en_US.UTF-8 vcs_info
    RPROMPT="${vcs_info_msg_0_}"
}
add-zsh-hook precmd _update_vcs_info_msg