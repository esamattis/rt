
# rt – "run task"

rt is a simple wrapper for task / scripts runners with instant tab-completion
for zsh.

Supported runners:

- package.json scripts
    - With auto detection for npm, pnpm, yarn 1.0
- Jake.js
- PHP composer.json scripts
- Picks excutables from `bin`, `scripts` and `tools` directories in the current
  working directory
    - Ex. to run `./scripts/build.sh` you can just type `rt build.sh`

The only reason this exists because tab-completion for npm scripts and jakefiles
is really slow. rt is written using Rust and [SWC](https://swc.rs/) for jakefile
parsing so it is basically instant. It also saves few key strokes!

But to be honest this project was just an excuse to learn Rust and SWC.

## Install

Get binary from the [releases page](https://github.com/esamattis/rt/releases)
and put it to PATH or build from the sources.

Enable tab-completion by putting this to  `~/.zshrc`:

```sh
compdef 'eval "$(rt --zsh-complete)"' rt
```


## Usage

In a project type `rt ` and hit the tab key

## Configuring

Set `RT_RUNNERS` environment variable to a comma separated list of runners you
want to use. List all available runners with `rt --runners`. By default all
runners are enabled.
