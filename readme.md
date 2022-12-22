# rust-shell
a unix shell written in rust
![screenshot of rust-shell](screenshots/2022-12-21_22-04.png)
## Features
* support all bash comments
* support PS1 variable (currently without commands, so just escapes like `\u`) - even something like `export PS1="\[\e[31m\][\[\e[m\]\[\e[38;5;172m\]\u\[\e[m\]@\[\e[38;5;153m\]\h\[\e[m\] \[\e[38;5;214m\]\W\[\e[m\]\[\e[31m\]]\[\e[m\]\\$ " # ]]]]]]]]]]`
* emacs editing mode ,like what `bash` have (using rustyline)
* fish-like autosuggestion
* support aliases (`alias ls=exa` and even `alias ls = 'ls --color=tty'`)
* support pipelines (like `cat file | grep finder`)
* support `export` variables and using them (including $Path)
* the prompt will updated with new pwd (like, when using `cd folder`, the prompt will be updated to `~/folder`).
* has `source` builtin, so you can run a file for set of commands.
* every time command exit with no-success, the `>` at the end of the prompt will be red, instead of green (present in the `$PS1` as `$red_or_green` environment variable)

# setup:
1. install `rustup` and `cargo`
2. Clone this repo (`git clone https://github.com/matan-h/rust-shell.git`)
3. run it with `cargo run`