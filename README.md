
# rt – "run task"

Instant ZSH autocompleter for npm package.json scripts and other task runners

Supported runners:

- package.json scripts
    - With auto detection for npm, pnpm, yarn 1.0
- Jake.js `jakefile.js` files
- PHP composer.json scripts
- Picks executables from `bin`, `scripts` and `tools` directories in the current
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

Set `RT_RUNNERS` environment variable to a comma separated list of runners without spaces you
want to use.

Available runners

 - `package.json`
 - `jakefile`
 - `composer.json`
 - `scripts:<dir>` - picks executables from `<dir>`. Ex. `scripts:bin`

 example

 ```sh
 export RT_RUNNERS=package.json,composer.json,scripts:./bin
 ```

List active runners with `rt --runners`

## Custom scripts

If you want to for example execute scripts from `node_modules/.bin` you can add

```
export RT_RUNNERS=scripts:node_modules/.bin
```

## Custom binaries

If you want to have for example different command for executing tasks and the `node_module/.bin` commands
you can add a symlink alias to your PATH and configure it with `<capitalized bin name>_RUNNERS`.
env.

Example

Create symlink alias

```
ln -s $(which rt) /usr/local/bin/rtn
```

and in your `~/.zshrc` add custom completer and config

```sh
export RTN_RUNNERS=scripts:node_modules/.bin
compdef 'eval "$(rtn --zsh-complete)"' rtn
```

Now you can use `rtn` for running `node_modules/.bin` commands and `rt` for running scripts from `package.json` and `composer.json`.
