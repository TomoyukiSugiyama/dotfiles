# dotfiles

## Installation

## Install Brew & Git

```bash
mkdir -p tmp/scripts \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/scripts/bootstrap.sh > tmp/scripts/bootstrap.sh \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/Brewfile > tmp/Brewfile \
&& cd tmp/scripts && chmod u+x bootstrap.sh && ./bootstrap.sh
```

## Using Git

```bash
git clone https://github.com/TomoyukiSugiyama/dotfiles.git && cd dotfiles/scripts && ./bootstrap.sh
```

## Link
```bash
cd scripts
./configuration.sh
```

Edit your `.gitconfig.local`.

```bash
vi ~/.gitconfig.local
```