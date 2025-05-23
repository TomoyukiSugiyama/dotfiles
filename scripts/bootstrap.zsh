#!/usr/bin/env zsh
# shellcheck disable=SC1091,SC2296

set -euo pipefail

script_path="${(%):-%N}"
script_dir="$(cd "$(dirname "${script_path}")" && pwd -P)"
dotdir=$(dirname "${script_dir}")

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
    brew upgrade --cask --greedy
    brew bundle --file "${dotdir}/Brewfile"
    if [ -f "${dotdir}/Brewfile.local" ]; then
        brew bundle --file "${dotdir}/Brewfile.local"
    fi
    brew doctor || true
}

function setup_gcloud_components() {
    # The next line updates PATH for the Google Cloud SDK.
    if [ -f "$(brew --prefix)/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/path.zsh.inc" ]; then
        source "$(brew --prefix)/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/path.zsh.inc"
    fi
    gcloud components install anthos-auth
    gcloud components install gke-gcloud-auth-plugin
}

function setup_helm_plugins(){
    if ! (type helm > /dev/null 2>&1); then
        echo "helm is not found in your local pc."
        return
    fi
    helm plugin install https://github.com/jkroepke/helm-secrets || true
    helm plugin install https://github.com/databus23/helm-diff || true
}

function setup_krew_plugins(){
    if ! (kubectl krew >/dev/null 2>&1); then
        echo "kubectl krew is not found in your local pc."
        return
    fi
    kubectl krew install view-secret
    kubectl krew install view-allocations
    kubectl krew install score
    kubectl krew install sniff
    kubectl krew install tree
    kubectl krew install resource-capacity
    kubectl krew upgrade
}

function setup_rust(){
    if ! (type rustup-init > /dev/null 2>&1); then
        echo "rustup-init is not found in your local pc."
        return
    fi
    if ! (type rustc > /dev/null 2>&1); then
        rustup-init -y
    fi
    rustup update
}

setup_brew
setup_gcloud_components
setup_helm_plugins
setup_rust
setup_krew_plugins

