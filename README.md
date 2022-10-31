# dotfiles

## Installation

## Install Brew & Git

```
mkdir -p tmp/scripts \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/scripts/bootstrap.sh > tmp/scripts/bootstrap.sh \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/Brewfile > tmp/Brewfile \
&& cd tmp/scripts && chmod u+x bootstrap.sh && ./bootstrap.sh
```

## Using Git

```
git clone https://github.com/TomoyukiSugiyama/dotfiles.git && cd dotfiles/scripts && ./bootstrap.sh
```
