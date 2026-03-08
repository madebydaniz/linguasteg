# Changelog

## [0.2.0](https://github.com/madebydaniz/linguasteg/compare/linguasteg-core-v0.1.0...linguasteg-core-v0.2.0) (2026-03-08)


### ✨ Features

* **analyze:** expose secure envelope metadata and inspection diagnostics ([38bbc12](https://github.com/madebydaniz/linguasteg/commit/38bbc12930bbbe1f6709039200c4018ad4473f35))
* **cli:** add farsi text-extractor decode path with trace fallback ([ce065ea](https://github.com/madebydaniz/linguasteg/commit/ce065ea717fbb581d3a4c99179e1832246776e40))
* **core:** add grammar realization templates and slot constraint planning abstractions ([7ef6256](https://github.com/madebydaniz/linguasteg/commit/7ef62567fb388ebbb4934912944d8bf23230bb94))
* **core:** add orchestration service for validation gateway routing and symbolic pipeline ([e7f9bbf](https://github.com/madebydaniz/linguasteg/commit/e7f9bbf53c105ee6ddb5572b35bb0dffdf988b21))
* **core:** add provider-agnostic model gateway contracts for orchestration ([5be5f4e](https://github.com/madebydaniz/linguasteg/commit/5be5f4e8f1f9e2fe4eb9cef03cb4074022f20d52))
* **core:** add style profile and reranking abstractions for author-inspired output ([426b183](https://github.com/madebydaniz/linguasteg/commit/426b183bf69c7e3bdea47225d4e1102bf977d71d))
* **core:** add symbolic frame payload decoding with schema validation ([a6347f5](https://github.com/madebydaniz/linguasteg/commit/a6347f53c45aa8099f4c29aa33bcfd808f7e77df))
* **core:** add symbolic payload planning for fixed-width slot frames ([eb66a5f](https://github.com/madebydaniz/linguasteg/commit/eb66a5f2d8d6d6b0ac7ed745fa3f7ad50f97ceb9))
* **core:** add text extractor contract and runtime wiring ([9c24f9c](https://github.com/madebydaniz/linguasteg/commit/9c24f9c6f3e69272249a4cc05f3cc1e440bec36d))
* **core:** add typed ids and registry contracts for language strategy model catalogs ([3c7be94](https://github.com/madebydaniz/linguasteg/commit/3c7be94ab5f60609e8d0075f01927e75d8503e7e))
* **core:** add versioned cryptographic envelope for keyed payload protection ([d288403](https://github.com/madebydaniz/linguasteg/commit/d288403f0cface09fcfabc168e125f1e117074bd))
* **core:** unify pipeline options and add decode request validation ([454505d](https://github.com/madebydaniz/linguasteg/commit/454505d4bbd21c509a8bb240ef6f5b6ebefbe641))
* **core:** validate encode requests against language strategy and model registries ([7fd66ba](https://github.com/madebydaniz/linguasteg/commit/7fd66ba5f7e71a8ab2d7f3a56cc279d05a96b335))


### 🐛 Bug Fixes

* **core,cli:** address review findings for dataset normalization, json escaping, and padding guards ([3d79218](https://github.com/madebydaniz/linguasteg/commit/3d7921872964cc0193c78b589f413029fab5db95))
* **release:** use explicit crate versions for release-please cargo-workspace compatibility ([2307fb4](https://github.com/madebydaniz/linguasteg/commit/2307fb442da92ea6a2ad015aa1b75befba018a6b))
* **security:** reduce codeql crypto false-positives in core and models ([65f57a8](https://github.com/madebydaniz/linguasteg/commit/65f57a80258798e1692936c4763d353c81833f4e))


### 🧱 Refactoring

* **crypto:** rename secret key material checks to reduce false-positive taint flow ([36ce3f0](https://github.com/madebydaniz/linguasteg/commit/36ce3f0fdc3e14315ab03d0b4d7ce240dc79ed7b))


### 🎨 Styles

* **core,models:** apply rustfmt normalization after Rust 1.85 update ([763bad9](https://github.com/madebydaniz/linguasteg/commit/763bad9e4ce5b6784476ea466c571d8ce896d7de))


### 👽 Miscellaneous

* **workspace:** bootstrap virtual workspace skeleton with facade crate ([5222c59](https://github.com/madebydaniz/linguasteg/commit/5222c590e0220b0e486dd4b1800356a577017657))
