#!/usr/bin/env zsh
# shellcheck disable=SC1090,SC1091

# --------------------------------------------------
# terminal
# --------------------------------------------------
export CLICOLOR=1 TERM=xterm-256color

autoload -Uz colors
colors

PROMPT=$'%F{blue}:%f %c/\n%# '

# vcs_info
autoload -Uz vcs_info
autoload -Uz add-zsh-hook
 
zstyle ':vcs_info:*' formats '%F{green}(%s)-[%b]%f'
zstyle ':vcs_info:*' actionformats '%F{red}(%s)-[%b|%a]%f'
zstyle ':vcs_info:*' enable git
 
function _update_vcs_info_msg() {
    LANG=en_US.UTF-8 vcs_info
    RPROMPT="${vcs_info_msg_0_}"
}

add-zsh-hook precmd _update_vcs_info_msg

if [ -f "$(brew --prefix)/share/kube-ps1.sh" ]; then
    ## if you want to customize, please edit `~/.kube/config`
    set +u
    export KUBECONFIG=${KUBECONFIG:-$HOME/.kube/config}
    export KUBE_PS1_CONTEXT=${KUBE_PS1_CONTEXT:-}
    export KUBE_PS1_ENABLED=true
    KUBE_PS1_SYMBOL_USE_IMG=true
    source "$(brew --prefix)/share/kube-ps1.sh"
    set -u
    PROMPT='$(kube_ps1) '"$PROMPT"
fi

# setopt
setopt no_beep
setopt nolistbeep

# history
HISTFILE="${HOME}/.zsh_history"
HISTSIZE=100000
SAVEHIST=1000000
setopt inc_append_history
setopt share_history

# --------------------------------------------------
# complement
# --------------------------------------------------
autoload -Uz compinit
compinit

zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# k8s
if type kubectl >/dev/null 2>&1; then
    source <(kubectl completion zsh)
fi

if type kwokctl >/dev/null 2>&1; then
    source <(kwokctl completion zsh)
    compdef _kwokctl kwokctl
fi

# helmfile
if type helmfile >/dev/null 2>&1; then
    source <(helmfile completion zsh)
fi

autoload -U +X bashcompinit && bashcompinit
complete -o nospace -C terraform terraform

# cloud-sql-proxy-v2-operator
if type cloud-sql-proxy-v2-operator >/dev/null 2>&1; then
    source <(cloud-sql-proxy-v2-operator completion zsh)
    compdef _cloud-sql-proxy-v2-operator cloud-sql-proxy-v2-operator
fi

# any-connect
if (type any-connect > /dev/null 2>&1); then
    source <(any-connect completion zsh)
    compdef _any-connect any-connect
fi

# The next line enables shell command completion for gcloud.
if [ -f "$(brew --prefix)/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/completion.zsh.inc" ]; then
    source "$(brew --prefix)/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/completion.zsh.inc"
fi

# istioctl
if (type istioctl > /dev/null 2>&1); then
    source <(istioctl completion zsh)
    compdef _istioctl istioctl
fi