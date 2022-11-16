#!/usr/bin/env zsh
# shellcheck disable=SC1090,SC1091,SC2034,SC2154

# terminal
CLICOLOR=1
TERM=xterm-256color

autoload -Uz colors
colors

PROMPT="%F{blue}[%m][%n]:%f %c/ %# "

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

# setopt
setopt no_beep
setopt nolistbeep

# history
HISTFILE="${HOME}/.zsh_history"
HISTSIZE=100000
SAVEHIST=100000

# complement
autoload -Uz compinit
compinit

zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# k8s
source <(kubectl completion zsh)

# helm
source <(helm completion zsh)

autoload -U +X bashcompinit && bashcompinit
complete -o nospace -C /opt/homebrew/Cellar/tfenv/3.0.0/versions/1.3.4/terraform terraform
