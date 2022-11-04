# shellcheck disable=SC1090,SC1091

export PATH="$PATH:/opt/homebrew/share/git-core/contrib/diff-highlight"
export PATH="/usr/local/sbin:$PATH"

# terminal
export CLICOLOR=1
export TERM=xterm-256color

autoload -U compinit
compinit

source "${HOME}/.zsh.alias.zsh"

# k8s
source <(kubectl completion zsh)
alias k=kubectl
compdef __start_kubectl k