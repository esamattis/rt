
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

## Install

Get binary from the [releases page](https://github.com/esamattis/rt/releases)
and put it to PATH or build from the sources.

Enable tab-completion by putting this to  `~/.zshrc`:

```sh
compdef 'eval "$(rt --zsh-complete $LBUFFER $RBUFFER)"' rt
```


## Usage

In a project type `rt ` and hit the tab key.

Just type `rt` to list available tasks and any possible errors.

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

## Custom Commands

If you want to have for example different command for executing tasks and the `node_module/.bin` commands.
You can use the `--runners-env` flag to configure the runners environment variable being read.

Example:

Use `rtn` execute `./node_modules/.bin/` commands:

```sh
export RTN_RUNNERS=scripts:node_modules/.bin
compdef 'eval "$(rt --runners-env RTN_RUNNERS --zsh-complete $LBUFFER $RBUFFER)"' rtn
rtn() {
    rt --runners-env RTN_RUNNERS $@
}
```

## Project Config

You may add a `.rtenv` file to your projects. When `rt` finds it from the current working it will read the environment variables from it.

Ex. `.rtenv`:

```sh
RT_RUNNERS=package.json,composer.json,scripts:./bin
```

## Fuzzy Matching

Combine with [fzf](https://github.com/junegunn/fzf) and [fzf-tab](https://github.com/Aloxaf/fzf-tab) for awesome fuzzy matching experience 🔥
