
# rt – "run task"

rt is a simple wrapper for JavaScript task runners with instant tab-completion
for zsh. It supports npm scripts and [Jake.js](https://jakejs.com/).

The only reason this exists because tab-completion for npm scripts and jakefiles
is really slow. rt is written using Rust and [SWC](https://swc.rs/) for jakefile
parsing so it is basically instant. It also saves few key strokes!

But to be honest this project was just an excuse to learn Rust and SWC.

## Install

todo

## Configure tab-completion

Add

```sh
compdef 'eval "$(rt --zsh-complete)"' rt
```

to `~/.zshrc`.


## Usage

In a project type `rt ` and hit the tab key