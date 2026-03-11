# linguasteg

Public facade crate for the LinguaSteg workspace.

## What this crate provides

`linguasteg` is the top-level import point that re-exports:
- Core contracts and pipeline primitives from `linguasteg-core`
- Prototype language packs and adapters from `linguasteg-models`

Use this crate when you want one stable dependency without importing internal workspace crates directly.

## Scope

This crate mainly provides re-exports. For end-user command-line usage, use `linguasteg-cli`.

## Workspace

- Repository: <https://github.com/madebydaniz/linguasteg>
- License: MIT
