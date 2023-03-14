# Development

## Debugging tips

- If a gtk or glib warning shows up in the terminal saying an assertion fail, set the environment variable `G_DEBUG=fatal_warnings` and run the app under `gdb`.
  It will crash when the assertion fails and you can inspect where that is in the backtrace.
