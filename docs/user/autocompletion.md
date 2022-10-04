# Autocompletion

One can generate autocompletion for its favorite shell using the following option:

```sh
cargo run -- completion --generator <GENERATOR>
```

Where `GENERATOR` can be any of `bash`, `elvish`, `fish`, `powershell` and `zsh`.

## Bash

First, get the completion file in a known place:

```sh
mkdir -p ~/.local/share/duniter
cargo run -- completion --generator bash > ~/.local/share/duniter/completion.bash
```

You can now manually source the file when needed:

```sh
source ~/.local/share/duniter/completion.bash
```

Or you can automatically source it at `bash` startup by adding this to your `~/.bashrc` file:

```sh
[[ -f $HOME/.local/share/duniter/completion.bash ]] && source $HOME/.local/share/duniter/completion.bash
```

You can now enjoy semantic completion of the `./target/release/duniter` command using `<Tab>` key.

## Zsh

Zsh equivalent

```sh
# make directory to store completion
mkdir -p ~/.zsh/completion
# write the completion script
cargo run -- completion --generator zsh > ~/.zsh/completion/_duniter.zsh
# add the following lines to your ~/.zshrc
fpath+=(~/.zsh/completion)
compinit # might slow down terminal startup
```
