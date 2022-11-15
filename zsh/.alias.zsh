#!/usr/bin/env zsh
# shellcheck disable=SC1090

# alias
alias ls='ls -G'

# Git
alias g='git'
alias gs='git status'
alias gd='git diff --cached'
alias gp='git pull --rebase'

# k8s
source <(kubectl completion zsh)
alias k=kubectl
compdef __start_kubectl k