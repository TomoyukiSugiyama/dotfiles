# dotfiles
Dotfiles for mac.

## Installation
These dotfiles are managed by brew, git and zsh scripts.
If you have git in your local pc, please install from 2nd step.
Otherwize, please install by 1st step.

### 1. Install git and minimal packages by using brew
Brew is installed by using `bootstrap.zsh`, if not exist in your local pc.
After that, Git and minimal packages are installed. 
If there is a package you want, please edit `Brewfile`.

```bash
mkdir -p tmp/scripts \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/scripts/bootstrap.zsh > tmp/scripts/bootstrap.zsh \
&& curl https://raw.githubusercontent.com/TomoyukiSugiyama/dotfiles/main/Brewfile > tmp/Brewfile \
&& chmod u+x ./tmp/scripts/bootstrap.zsh && ./tmp/scripts/bootstrap.zsh
```

If you finish installation, please delete all files in `tmp` dir and after that, managed by git.

### 2. Install minimal packages by using brew and git
You can use anywhere.

```bash
git clone https://github.com/TomoyukiSugiyama/dotfiles.git && ./dotfiles/scripts/bootstrap.zsh
```

### 3. Configuration
Each dotfiles (in git, zsh and etc. dirs) are linked to home dir by using `configuration.zsh`.

```bash
./scripts/configuration.zsh
```

Edit your `.gitconfig.local` and set name and email.

```bash
vi ~/.gitconfig.local
```