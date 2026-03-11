# linguasteg-cli

Command-line interface for LinguaSteg (`lsteg`).

## Install

```bash
cargo install linguasteg-cli --locked
```

## Quick start

```bash
lsteg --version
lsteg languages

lsteg encode --lang en --message "hello world" --secret "test-secret"
lsteg decode --lang en --trace-input --trace "<encoded-text-or-trace>" --secret "test-secret"
```

## Data management

```bash
lsteg data list --lang en
lsteg data install --lang en --download
lsteg data status --lang en
```

## Notes

- Use the same `--secret` value for both encode and decode.
- You can provide input and output files with `--input` and `--output`.
- For full command usage, run `lsteg --help`.

## Workspace

- Repository: <https://github.com/madebydaniz/linguasteg>
- License: MIT
