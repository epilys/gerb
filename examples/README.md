# REPL

This is a python interpreter that can load and inspect a UFO dir using gerb's python API.

You can run it as:

```shell
cargo run
```

or

```shell
cargo run /path/to/dir.ufo
```

Using `rlwrap`, a tool that wraps commands with the readline interface, allows you to have history, use backspace etc.

```shell
rlwrap -f examples/.py_completion cargo run --example repl -- /path/to/dir.ufo
```
