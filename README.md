<div align="center">
  <img src="assets/logo.png" alt="LinguaSteg Logo" width="220">
  <h1>LinguaSteg</h1>
</div>

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/madebydaniz/linguasteg)](https://github.com/madebydaniz/linguasteg/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/madebydaniz/linguasteg/ci-main-gate.yml?branch=develop)](https://github.com/madebydaniz/linguasteg/actions/workflows/ci-main-gate.yml)
[![Release Binaries](https://img.shields.io/github/actions/workflow/status/madebydaniz/linguasteg/release-binaries.yml?branch=main&label=release-binaries)](https://github.com/madebydaniz/linguasteg/actions/workflows/release-binaries.yml)

Multilingual linguistic steganography CLI in Rust.

Encode secret-protected payloads into natural-language cover text and decode them back deterministically.

</div>

## Installation/Update

### Homebrew

```bash
brew tap madebydaniz/tap
brew install lsteg
```

Update:

```bash
brew update
brew upgrade lsteg
```

### Install script

```bash
curl -fsSL https://raw.githubusercontent.com/madebydaniz/linguasteg/main/scripts/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/madebydaniz/linguasteg/main/scripts/install.sh | bash -s -- --version v0.2.0
```

Note:

- Script checksum verification is always enabled.
- Signature verification uses Cosign keyless by default.
- To skip signature verification (not recommended): `--no-verify-signature`

### Build from source

```bash
git clone https://github.com/madebydaniz/linguasteg.git
cd linguasteg
cargo install --path linguasteg-cli --locked
```

## Usage

### Quick start

```bash
# Install dataset
lsteg data install --lang en --download --format json

# Encode
lsteg encode --lang en --message "hello world" --secret "test-secret" --format text

# Decode
lsteg decode --lang auto --text-input --trace "<stego text>" --secret "test-secret" --format json
```

### Main commands

| Command                              | Example                                                              |
| ------------------------------------ | -------------------------------------------------------------------- |
| `encode`                             | `lsteg encode --lang fa --message "salam" --secret "k"`              |
| `decode`                             | `lsteg decode --lang auto --text-input --trace "..." --secret "k"`   |
| `analyze`                            | `lsteg analyze --lang auto --text-input --trace "..." --format json` |
| `validate`                           | `lsteg validate --lang auto --trace-input --trace "..."`             |
| `catalog`                            | `lsteg catalog --format json`                                        |
| `templates` / `profiles` / `schemas` | `lsteg templates --lang de`                                          |
| `data install`                       | `lsteg data install --lang all --download`                           |
| `data list`                          | `lsteg data install --lang en --source list --format json`           |
| `data status`                        | `lsteg data status --format json`                                    |
| `data update`                        | `lsteg data update --lang it --download`                             |
| `data verify`                        | `lsteg data verify --lang en --source en-wordnet-princeton`          |

## Release Signature Verification (Cosign)

Release integrity is enforced with keyless Cosign signatures over `checksums.txt`, and those checksums are used to verify every published binary archive.

Every GitHub Release publishes:

- `checksums.txt`
- `checksums.txt.sig`
- `checksums.txt.pem`
- `checksums.txt.bundle`

Example verification:

```bash
VERSION=v0.2.0
curl -fsSLO "https://github.com/madebydaniz/linguasteg/releases/download/${VERSION}/checksums.txt"
curl -fsSLO "https://github.com/madebydaniz/linguasteg/releases/download/${VERSION}/checksums.txt.bundle"

cosign verify-blob \
  --bundle checksums.txt.bundle \
  --certificate-identity-regexp "^https://github.com/madebydaniz/linguasteg/\\.github/workflows/release-binaries\\.yml@refs/(heads/main|tags/.+)$" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  checksums.txt
```

## Supported Languages

| Code | Language | Direction |
| ---- | -------- | --------- |
| `fa` | Farsi    | `rtl`     |
| `en` | English  | `ltr`     |
| `de` | German   | `ltr`     |
| `it` | Italian  | `ltr`     |

## License

Licensed under the **MIT License** - see [LICENSE](LICENSE) file for details.

## Support & Community

- **Issues & Bug Reports**: [GitHub Issues](https://github.com/madebydaniz/linguasteg/issues)

## Changelog

- [`linguasteg/CHANGELOG.md`](linguasteg/CHANGELOG.md)
- [`linguasteg-cli/CHANGELOG.md`](linguasteg-cli/CHANGELOG.md)
- [`linguasteg-core/CHANGELOG.md`](linguasteg-core/CHANGELOG.md)
- [`linguasteg-models/CHANGELOG.md`](linguasteg-models/CHANGELOG.md)
- [`linguasteg-eval/CHANGELOG.md`](linguasteg-eval/CHANGELOG.md)

## Authors

**LinguaSteg** is maintained by [Daniel Niazmand](https://github.com/madebydaniz) with support from the community.

---

<div align="center">

**Made with ❤️ for open-source builders and privacy-first communication.**

[GitHub](https://github.com/madebydaniz/linguasteg) • [Report Bug](https://github.com/madebydaniz/linguasteg/issues) • [Request Feature](https://github.com/madebydaniz/linguasteg/issues)

</div>
