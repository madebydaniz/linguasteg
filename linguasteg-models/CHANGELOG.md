# Changelog

## [0.2.0](https://github.com/madebydaniz/linguasteg/compare/linguasteg-models-v0.1.0...linguasteg-models-v0.2.0) (2026-03-08)


### ✨ Features

* **cli,models:** add english prototype scaffold with unified multi-language runtime ([b99a3bb](https://github.com/madebydaniz/linguasteg/commit/b99a3bbc5369822f908b62f805cc2e2b10a37e75))
* **cli,models:** diversify secret-based text surfaces with reversible alias parsing ([203c147](https://github.com/madebydaniz/linguasteg/commit/203c1477829a01272f53197e8322c6cb1c943f46))
* **cli:** add farsi text-extractor decode path with trace fallback ([ce065ea](https://github.com/madebydaniz/linguasteg/commit/ce065ea717fbb581d3a4c99179e1832246776e40))
* **cli:** wire encode profile selection through runtime and farsi realization ([cf6b270](https://github.com/madebydaniz/linguasteg/commit/cf6b270c2294bc3121fe31c39729d47517ff53a2))
* **cli:** wire italian runtime, data source, and trace detection ([e8fc952](https://github.com/madebydaniz/linguasteg/commit/e8fc9526b3c553a3c0970bb2f9b3718b42584ed3))
* **core:** add text extractor contract and runtime wiring ([9c24f9c](https://github.com/madebydaniz/linguasteg/commit/9c24f9c6f3e69272249a4cc05f3cc1e440bec36d))
* **en:** add author-inspired style profiles with reversible lexical variants ([35fbcaa](https://github.com/madebydaniz/linguasteg/commit/35fbcaa64ad1c5b626d736a68213e07648527039))
* **en:** add decode-safe verb aliasing for author-inspired profiles ([f7261ac](https://github.com/madebydaniz/linguasteg/commit/f7261ac5e46eb763efb96bbf523eba05c5284dab))
* **en:** enable lossless plain-text decode via extractor and reverse mapping ([e83c1e4](https://github.com/madebydaniz/linguasteg/commit/e83c1e43f59bf2c88ce248ee9678badf56127bbc))
* **fa:** enable lossless plain-text decode with full-width symbolic inventories ([0d11dd9](https://github.com/madebydaniz/linguasteg/commit/0d11dd951e14cd426562a829779987500b2fd282))
* **it:** localize italian prototype lexicon, parser, and starter dataset ([2ad6c52](https://github.com/madebydaniz/linguasteg/commit/2ad6c5268b2e85cca085e2cdf13476941c2ce711))
* **models:** add Farsi lexical inventories and slot compatibility checks ([a706760](https://github.com/madebydaniz/linguasteg/commit/a706760d9f7a8267cb3b2eb6b24d5455c877a854))
* **models:** add Farsi prototype language pack with templates styles and basic constraints ([18a1c8b](https://github.com/madebydaniz/linguasteg/commit/18a1c8b9427fd03af1c8498604a5c585594edcab))
* **models:** add Farsi symbolic mapper from slot frames to realization plans ([0bc6806](https://github.com/madebydaniz/linguasteg/commit/0bc6806171b4191ace8dfdd9343d808a96731a00))
* **models:** add reverse symbolic mapping and payload decode for farsi plans ([2cfee32](https://github.com/madebydaniz/linguasteg/commit/2cfee32abbed8d45d9d10065080553b12ba32d02))
* **models:** add stub gateway adapter and in-memory gateway registry ([8078859](https://github.com/madebydaniz/linguasteg/commit/807885991da2bf28f1fd7610547d35d203e95413))
* **models:** enrich farsi literary profile surfaces with reversible lexicon aliases ([d8249ea](https://github.com/madebydaniz/linguasteg/commit/d8249eae9c64f60b2d0067e29afffb94ad3052da))
* **models:** improve english output naturalness with backward-compatible object aliases ([758c2e8](https://github.com/madebydaniz/linguasteg/commit/758c2e8e425ff477e11b5dae9b67c1c99419253e))
* **models:** scaffold german language pack and runtime registration ([4935f52](https://github.com/madebydaniz/linguasteg/commit/4935f520c3adcca6ae52e651e4946f47e1338189))
* **models:** scaffold italian language module via xtask ([f75eab5](https://github.com/madebydaniz/linguasteg/commit/f75eab5fb9cae2a10b04553bf8cfce39b318af1b))


### 🐛 Bug Fixes

* **cargo:** stabilize workspace internal crate versioning for release please ([e11fc8c](https://github.com/madebydaniz/linguasteg/commit/e11fc8c46b66fa8dd25c51e268551c5a557b153f))
* **cargo:** stabilize workspace internal crate versioning for release-please ([384ae31](https://github.com/madebydaniz/linguasteg/commit/384ae3141a76be8a231b8a4e9a4f6cf14909141a))
* **models:** remove codeql hard-coded-crypto false positives from italian profile gating ([18df935](https://github.com/madebydaniz/linguasteg/commit/18df935ab575dccf0d8591777e28f0f8787a5540))
* **models:** remove const-empty lexicon check from noun selection ([5afdd43](https://github.com/madebydaniz/linguasteg/commit/5afdd43181002403a77b178d8f32dea895159974))
* **models:** resolve italian codeql false positives and refine release changelog sections ([9bbe685](https://github.com/madebydaniz/linguasteg/commit/9bbe685e1bb0cebb5a2e76026f333c3aab6befd3))
* **release:** use explicit crate versions for release-please cargo-workspace compatibility ([2307fb4](https://github.com/madebydaniz/linguasteg/commit/2307fb442da92ea6a2ad015aa1b75befba018a6b))
* **security:** reduce codeql crypto false-positives in core and models ([65f57a8](https://github.com/madebydaniz/linguasteg/commit/65f57a80258798e1692936c4763d353c81833f4e))


### 🧱 Refactoring

* **cli:** remove gateway stub output from encode/decode reports ([948e75c](https://github.com/madebydaniz/linguasteg/commit/948e75cce829c7201703088e7a0b1bd7ab96e901))


### 🎨 Styles

* **core,models:** apply rustfmt normalization after Rust 1.85 update ([763bad9](https://github.com/madebydaniz/linguasteg/commit/763bad9e4ce5b6784476ea466c571d8ce896d7de))


### 👽 Miscellaneous

* **workspace:** bootstrap virtual workspace skeleton with facade crate ([5222c59](https://github.com/madebydaniz/linguasteg/commit/5222c590e0220b0e486dd4b1800356a577017657))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * linguasteg-core bumped from 0.1.0 to 0.2.0
