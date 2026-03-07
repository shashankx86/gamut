# Logging

`gamut` uses the Rust `log` facade with `env_logger` for runtime logging.

## Default behavior

- The default log level is `info`.
- This keeps normal runs visible enough for operations while avoiding debug noise.
- Log output is only initialized for runtime commands, not for `--help` or `--version`.

## Override with `RUST_LOG`

`env_logger` reads the standard `RUST_LOG` environment variable. If it is set, it overrides the default `info` filter.

## Examples

```bash
gamut --toggle
RUST_LOG=warn gamut --daemon
RUST_LOG=debug gamut --toggle
RUST_LOG=trace gamut --quit
```

## Guidance

- Use `info` for normal operational milestones.
- Use `warn` for recoverable problems or degraded states.
- Use `error` for failures.
- Use `debug` or `trace` only when investigating issues.
