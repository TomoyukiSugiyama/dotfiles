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
