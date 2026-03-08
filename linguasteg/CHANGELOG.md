# Changelog

## [0.2.0](https://github.com/madebydaniz/linguasteg/compare/linguasteg-v0.1.0...linguasteg-v0.2.0) (2026-03-08)


### ✨ Features

* **core:** add grammar realization templates and slot constraint planning abstractions ([7ef6256](https://github.com/madebydaniz/linguasteg/commit/7ef62567fb388ebbb4934912944d8bf23230bb94))
* **core:** add orchestration service for validation gateway routing and symbolic pipeline ([e7f9bbf](https://github.com/madebydaniz/linguasteg/commit/e7f9bbf53c105ee6ddb5572b35bb0dffdf988b21))
* **core:** add provider-agnostic model gateway contracts for orchestration ([5be5f4e](https://github.com/madebydaniz/linguasteg/commit/5be5f4e8f1f9e2fe4eb9cef03cb4074022f20d52))
* **core:** add style profile and reranking abstractions for author-inspired output ([426b183](https://github.com/madebydaniz/linguasteg/commit/426b183bf69c7e3bdea47225d4e1102bf977d71d))
* **core:** add symbolic frame payload decoding with schema validation ([a6347f5](https://github.com/madebydaniz/linguasteg/commit/a6347f53c45aa8099f4c29aa33bcfd808f7e77df))
* **core:** add symbolic payload planning for fixed-width slot frames ([eb66a5f](https://github.com/madebydaniz/linguasteg/commit/eb66a5f2d8d6d6b0ac7ed745fa3f7ad50f97ceb9))
* **core:** add typed ids and registry contracts for language strategy model catalogs ([3c7be94](https://github.com/madebydaniz/linguasteg/commit/3c7be94ab5f60609e8d0075f01927e75d8503e7e))
* **core:** unify pipeline options and add decode request validation ([454505d](https://github.com/madebydaniz/linguasteg/commit/454505d4bbd21c509a8bb240ef6f5b6ebefbe641))
* **core:** validate encode requests against language strategy and model registries ([7fd66ba](https://github.com/madebydaniz/linguasteg/commit/7fd66ba5f7e71a8ab2d7f3a56cc279d05a96b335))
* **models:** add Farsi lexical inventories and slot compatibility checks ([a706760](https://github.com/madebydaniz/linguasteg/commit/a706760d9f7a8267cb3b2eb6b24d5455c877a854))
* **models:** add Farsi prototype language pack with templates styles and basic constraints ([18a1c8b](https://github.com/madebydaniz/linguasteg/commit/18a1c8b9427fd03af1c8498604a5c585594edcab))
* **models:** add Farsi symbolic mapper from slot frames to realization plans ([0bc6806](https://github.com/madebydaniz/linguasteg/commit/0bc6806171b4191ace8dfdd9343d808a96731a00))
* **models:** add stub gateway adapter and in-memory gateway registry ([8078859](https://github.com/madebydaniz/linguasteg/commit/807885991da2bf28f1fd7610547d35d203e95413))
* **models:** scaffold german language pack and runtime registration ([4935f52](https://github.com/madebydaniz/linguasteg/commit/4935f520c3adcca6ae52e651e4946f47e1338189))
* **models:** scaffold italian language module via xtask ([f75eab5](https://github.com/madebydaniz/linguasteg/commit/f75eab5fb9cae2a10b04553bf8cfce39b318af1b))


### 🐛 Bug Fixes

* **cargo:** stabilize workspace internal crate versioning for release please ([e11fc8c](https://github.com/madebydaniz/linguasteg/commit/e11fc8c46b66fa8dd25c51e268551c5a557b153f))
* **cargo:** stabilize workspace internal crate versioning for release-please ([384ae31](https://github.com/madebydaniz/linguasteg/commit/384ae3141a76be8a231b8a4e9a4f6cf14909141a))
* **release:** use explicit crate versions for release-please cargo-workspace compatibility ([2307fb4](https://github.com/madebydaniz/linguasteg/commit/2307fb442da92ea6a2ad015aa1b75befba018a6b))


### 👽 Miscellaneous

* **workspace:** bootstrap virtual workspace skeleton with facade crate ([5222c59](https://github.com/madebydaniz/linguasteg/commit/5222c590e0220b0e486dd4b1800356a577017657))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * linguasteg-core bumped from 0.1.0 to 0.2.0
    * linguasteg-models bumped from 0.1.0 to 0.2.0
