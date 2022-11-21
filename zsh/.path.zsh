#!/usr/bin/env zsh

export PATH="$PATH:/opt/homebrew/share/git-core/contrib/diff-highlight"
export PATH="/usr/local/sbin:$PATH"

# The next line updates PATH for the Google Cloud SDK.
if [ -f "${HOME}/google-cloud-sdk/path.zsh.inc" ]; then . "${HOME}/google-cloud-sdk/path.zsh.inc"; fi
