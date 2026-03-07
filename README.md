# LinguaSteg

LinguaSteg is an enterprise-oriented Rust workspace for multilingual linguistic steganography.

It provides:
- a reusable domain core (`linguasteg-core`)
- language/model packs (`linguasteg-models`)
- a public facade crate (`linguasteg`)
- a production-focused CLI (`linguasteg-cli`, binary: `lsteg`)

## Status

- Version: `0.1.0`
- Rust edition: `2024`
- MSRV: `1.85`
- Current implemented languages: `fa`, `en`, `de`, `it`
- Current delivery focus: release-candidate hardening on `develop`

## What It Does

LinguaSteg encodes input text into symbolic frames and realizes those frames as natural-language cover text.
The same cover text can later be decoded back (lossless in supported canonical paths) with secret-aware envelope protection.

Core capabilities currently available:
- secret-aware encode/decode pipeline
- trace analysis and validation
- text and JSON contracts for automation
- multilingual runtime catalog discovery (`languages`, `models`, `strategies`, `templates`, `profiles`, `schemas`, `catalog`)
- dataset lifecycle commands for lexicon/source management

## Workspace Structure

```text
linguasteg/
├── linguasteg             # facade crate (re-exports core + models APIs)
├── linguasteg-core        # domain contracts, symbolic engine, orchestration, crypto envelope
├── linguasteg-models      # language packs, text extractors, mapper/realizer/gateway adapters
├── linguasteg-cli         # lsteg command-line interface
├── linguasteg-eval        # evaluation/metrics scaffold
├── docs                   # governance, release and policy documents
└── scripts                # CI/release helper scripts
```

## Installation and Local Build

Clone and build:

```bash
git clone https://github.com/madebydaniz/linguasteg.git
cd linguasteg
cargo check --workspace --all-targets --all-features --locked
```

Run tests:

```bash
cargo test --workspace --all-features --locked
```

Run CLI:

```bash
cargo run -q -p linguasteg-cli -- --help
```

Optional local install of `lsteg` binary:

```bash
cargo install --path linguasteg-cli --locked
```

## CLI Quick Reference

Main commands:

```text
encode | decode | analyze | validate
languages | strategies | models | catalog | templates | profiles | schemas
data (install/update/list/status/verify/pin/export-manifest/import-manifest/doctor/clean/artifact validate)
demo | proto-encode | proto-decode
```

Get complete usage:

```bash
lsteg --help
```

## Core Usage Examples

### 1) First-run flow (recommended)

```bash
lsteg data install --lang en --download --format json
lsteg encode --lang en --message "hello world" --secret "test-secret" --format text
```

### 2) Encode and decode with a secret

```bash
lsteg encode \
  --lang en \
  --message "hello world" \
  --secret "test-secret" \
  --format json
```

Then decode from generated stego text or trace:

```bash
lsteg decode \
  --lang auto \
  --text-input \
  --trace "<stego text>" \
  --secret "test-secret" \
  --format json
```

### 3) Analyze or validate input integrity

```bash
lsteg analyze --lang auto --trace-input --trace "<trace>" --format json
lsteg validate --lang auto --trace-input --trace "<trace>" --format json
```

### 4) Inspect runtime capabilities

```bash
lsteg catalog --format json
lsteg templates --lang en --format text
lsteg profiles --lang fa --format text
lsteg schemas --lang en --format json
```

### 5) Work with dataset sources

Install one or more language sources with default downloadable artifacts:

```bash
lsteg data install --lang en --download --data-dir ./data --format json
lsteg data install --lang all --download --data-dir ./data --format json
```

This creates a starter dataset file at:

```text
./data/en/<source-id>/dataset.json
```

The installed dataset artifact becomes active immediately.
To discover source IDs for a specific language:

```bash
lsteg data install --lang en --source list --format json
```

Edit starter file any time, then refresh activation without providing URL:

```bash
lsteg data update --lang en --source en-wordnet-princeton --download --data-dir ./data --format json
```

Optional explicit artifact import (file/http URL) is still supported:

```bash
lsteg data install --lang en --source en-wordlist-wordnik --artifact-url file:///path/to/artifact.json --data-dir ./data --format json
```

Check health and integrity:

```bash
lsteg data status --lang en --data-dir ./data --format json
lsteg data verify --lang en --source en-wordlist-wordnik --data-dir ./data --format json
```

Validate a dataset artifact contract:

```bash
lsteg data artifact validate --lang en --input file:///path/to/artifact.json --format json
```

## Environment Variables

The CLI supports env defaults. CLI flags override env values.

- `LSTEG_LANG`
- `LSTEG_FORMAT`
- `LSTEG_INPUT`
- `LSTEG_OUTPUT`
- `LSTEG_ENCODE_MESSAGE`
- `LSTEG_PROFILE`
- `LSTEG_TRACE`
- `LSTEG_SECRET`
- `LSTEG_SECRET_FILE`
- `LSTEG_DATA_DIR`

## Using LinguaSteg as a Library

Yes, you can integrate only `linguasteg-core` in other apps.

Typical integration choices:
- `linguasteg-core`: when you want only domain contracts, symbolic planning, validation, crypto envelope, and orchestration abstractions.
- `linguasteg-core` + `linguasteg-models`: when you also want ready language packs/adapters (current `fa`/`en`/`de`/`it` implementations).
- `linguasteg` (facade): when you prefer a single dependency re-exporting core + models public APIs.

Example path dependency setup:

```toml
[dependencies]
linguasteg-core = { path = "../linguasteg/linguasteg-core" }
```

## Engineering and Release Gates

This repository enforces release-grade checks and governance:
- strict quality gate (check, clippy bug-risk profile, tests)
- dependency review
- cargo-audit
- CodeQL
- release readiness checks
- release drill workflow
- release-please automation
- crates publish workflow on release events

Useful local scripts:

```bash
./scripts/ci/pre_release_checklist.sh fast
./scripts/ci/pre_release_checklist.sh full
./scripts/ci/go_no_go_check.sh report
./scripts/ci/go_no_go_check.sh strict
```

## Contributing

1. Branch from `develop`.
2. Keep commits small and use Conventional Commits.
3. Run quality gates locally.
4. Open PR to `develop` (or `main` only for approved release promotion).

Please avoid committing private planning files:
- `task_plan.md`
- `findings.md`
- `progress.md`

## Security Notes

- Secrets are required for secure encode/decode flows.
- Treat generated traces and stego text as potentially sensitive.
- Never commit real secrets, production payloads, or private corpora to the repository.

## License

MIT. See `LICENSE`.
