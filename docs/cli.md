# CLI

`gamut` supports a small set of command-line flags for local control and inspection.

## Usage

```text
gamut [OPTIONS]
```

## Options

- `-h`, `--help` - print the ASCII banner followed by the available options.
- `-v`, `--version` - print the current release as `gamut <version>`.
- `--toggle` - toggle the launcher. This is the default behavior when no mode flag is provided.
- `--daemon` - start the background daemon process.
- `--quit` - send a quit request to the running daemon.

## Notes

- `--help` takes precedence over other flags.
- `--version` takes precedence over mode flags such as `--daemon` and `--quit`.
- When multiple mode flags are provided, the last mode flag wins.

## Examples

```bash
gamut
gamut --toggle
gamut --daemon
gamut --quit
gamut --help
gamut --version
```
