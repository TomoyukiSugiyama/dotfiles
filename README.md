# dotfiles

## Installation

## Using curl

```
mkdir dotfiles \
&& curl -#L https://github.com/TomoyukiSugiyama/dotfiles/tarball/main | tar -xzv --strip-components 1 -C dotfiles \
&& cd dotfiles/scripts \
&& ./bootstrap.sh
```

## Using Git

```
git clone https://github.com/TomoyukiSugiyama/dotfiles.git && cd dotfiles/scripts && ./bootstrap.sh
```
