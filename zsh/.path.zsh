#!/usr/bin/env zsh
# shellcheck disable=SC1091

export PATH="$PATH:/opt/homebrew/share/git-core/contrib/diff-highlight"
export PATH="/usr/local/sbin:$PATH"

# The next line updates PATH for the Google Cloud SDK.
if [ -f "$(brew --prefix)/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/path.zsh.inc" ]; then
    source "$(brew --prefix)/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/path.zsh.inc"
fi

# gcloud.
export USE_GKE_GCLOUD_AUTH_PLUGIN=True

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# asdf
if [ -f "/opt/homebrew/opt/asdf/libexec/asdf.sh" ]; then
    source "$(brew --prefix)/opt/asdf/libexec/asdf.sh"
fi

# krew
export PATH="${PATH}:${HOME}/.krew/bin"