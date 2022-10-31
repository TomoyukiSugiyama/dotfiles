# dotfiles

## Installation

### Install Brew & Git

```bash
mkdir -p tmp/scripts \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/scripts/bootstrap.zsh > tmp/scripts/bootstrap.zsh \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/Brewfile > tmp/Brewfile \
&& chmod u+x ./tmp/scripts/bootstrap.zsh && ./tmp/scripts/bootstrap.zsh
```

### Using Git

```bash
git clone https://github.com/TomoyukiSugiyama/dotfiles.git && ./dotfiles/scripts/bootstrap.zsh
```

### Configuration

```bash
./scripts/configuration.zsh
```

Edit your `.gitconfig.local`.

```bash
vi ~/.gitconfig.local
```