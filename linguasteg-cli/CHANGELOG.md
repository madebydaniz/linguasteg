# Changelog

## [0.2.0](https://github.com/madebydaniz/linguasteg/compare/linguasteg-cli-v0.1.0...linguasteg-cli-v0.2.0) (2026-03-08)


### ✨ Features

* **analyze:** expose secure envelope metadata and inspection diagnostics ([38bbc12](https://github.com/madebydaniz/linguasteg/commit/38bbc12930bbbe1f6709039200c4018ad4473f35))
* **cli,models:** add english prototype scaffold with unified multi-language runtime ([b99a3bb](https://github.com/madebydaniz/linguasteg/commit/b99a3bbc5369822f908b62f805cc2e2b10a37e75))
* **cli,models:** diversify secret-based text surfaces with reversible alias parsing ([203c147](https://github.com/madebydaniz/linguasteg/commit/203c1477829a01272f53197e8322c6cb1c943f46))
* **cli:** add auto language detection for decode and analyze traces ([2f030cf](https://github.com/madebydaniz/linguasteg/commit/2f030cf620b651067c1a6818cf5dabe577dfdde9))
* **cli:** add catalog command for unified languages strategies and models discovery ([18fdab6](https://github.com/madebydaniz/linguasteg/commit/18fdab6771086b7f66bc21254198d8a0559bf7e0))
* **cli:** add data artifact validate subcommand for lexicon datasets ([efb033f](https://github.com/madebydaniz/linguasteg/commit/efb033fdc37972f5733140a9272ef034d50cafdf))
* **cli:** add data clean command with preview and apply modes ([c433da9](https://github.com/madebydaniz/linguasteg/commit/c433da9277d522446fb71c231f7b871a0d6ba2f4))
* **cli:** add data doctor command with optional self-heal mode ([8352a7f](https://github.com/madebydaniz/linguasteg/commit/8352a7f1f7353f07936a892279004ad1464593e7))
* **cli:** add data export-manifest command for reproducible snapshots ([ac27af9](https://github.com/madebydaniz/linguasteg/commit/ac27af92433394ebd597387497ee6f2b39079ef7))
* **cli:** add data import-manifest command for snapshot restore ([81f0a00](https://github.com/madebydaniz/linguasteg/commit/81f0a00b399a03998bfa0cc4a3d74eb1b69297ff))
* **cli:** add data lifecycle commands with local cache state and manifests ([9f75e6d](https://github.com/madebydaniz/linguasteg/commit/9f75e6d7ad6f224cdb125aa8a97ebc71e5a789e3))
* **cli:** add data pin command for checksum freeze in local manifests ([d0e7119](https://github.com/madebydaniz/linguasteg/commit/d0e7119f67f381db5c8ee114d4c02b230e64f822))
* **cli:** add data status command for manifest and artifact health ([79e1b38](https://github.com/madebydaniz/linguasteg/commit/79e1b3886edccf54d506478fc1bad1f506ce2c83))
* **cli:** add data verify command for dataset integrity checks ([170e32e](https://github.com/madebydaniz/linguasteg/commit/170e32e42c546531ffb4754d73d5d9b1253c2551))
* **cli:** add default de data source and dynamic data language discovery ([1467537](https://github.com/madebydaniz/linguasteg/commit/14675372f7b41abb4943c9ff027bfaa1016149f2))
* **cli:** add downloadable dataset UX and lang-all install flow ([155acc9](https://github.com/madebydaniz/linguasteg/commit/155acc915c36aa4a702917db60bafe9e650e2fa2))
* **cli:** add emit-trace flag and explicit decode input modes ([539ff29](https://github.com/madebydaniz/linguasteg/commit/539ff29138b3737e3cbe77b5e3214aac5bb7929d))
* **cli:** add env-based config layering with cli override precedence ([0642181](https://github.com/madebydaniz/linguasteg/commit/0642181aa3bd915b2c29d7159f61e2a2c08d0aee))
* **cli:** add farsi first-frame secret-sensitive intro variants ([c8bb7a0](https://github.com/madebydaniz/linguasteg/commit/c8bb7a0298740fb843ac3cb3dfb12da7a21d07a9))
* **cli:** add farsi proto-decode from frame trace input ([218bc91](https://github.com/madebydaniz/linguasteg/commit/218bc91733c2187ff129ad174d984fcc09c98d7e))
* **cli:** add farsi prototype demo command for rendering and validation ([6fdb066](https://github.com/madebydaniz/linguasteg/commit/6fdb066899bb83dae9bd465dfa6449fe065546a0))
* **cli:** add farsi prototype end-to-end encode command ([849b89b](https://github.com/madebydaniz/linguasteg/commit/849b89bfebbd4a73e87d3650e557f09ec249e093))
* **cli:** add farsi text-extractor decode path with trace fallback ([ce065ea](https://github.com/madebydaniz/linguasteg/commit/ce065ea717fbb581d3a4c99179e1832246776e40))
* **cli:** add json output mode for proto encode and decode ([1043205](https://github.com/madebydaniz/linguasteg/commit/104320532ae31451bc6669b17a8732fc5795d9b6))
* **cli:** add languages command with runtime-backed metadata and json contract ([3037296](https://github.com/madebydaniz/linguasteg/commit/3037296fb78c07eb3895b6437c04a08c972dd879))
* **cli:** add LSTEG_SECRET_FILE env support with secret-source hardening ([f13a13f](https://github.com/madebydaniz/linguasteg/commit/f13a13fa64dc715d87993bdf625aa411444764ab))
* **cli:** add models command with text/json metadata output ([a138a4b](https://github.com/madebydaniz/linguasteg/commit/a138a4b722b9edc04be489e0715c62e4ba1938b1))
* **cli:** add profiles command with style metadata and language filter ([4e32083](https://github.com/madebydaniz/linguasteg/commit/4e32083913d104e15fc1c81431953d98822c4627))
* **cli:** add schemas command for symbolic frame introspection ([0c33eda](https://github.com/madebydaniz/linguasteg/commit/0c33eda17050ef57499c9b5d0120dcdff550636f))
* **cli:** add strategies command with text/json metadata output ([bf9b50d](https://github.com/madebydaniz/linguasteg/commit/bf9b50dce1bcf4fa67083cf8fdcc0caa8d0d9ea1))
* **cli:** add style_profile field to proto-encode json contract ([38dd432](https://github.com/madebydaniz/linguasteg/commit/38dd43228eff8a28692792b6aece0d6aa14cda3e))
* **cli:** add templates command with language filter and metadata output ([7091188](https://github.com/madebydaniz/linguasteg/commit/709118883032de0fcf8aab5f3e3bdf25223df14a))
* **cli:** add text/trace input modes to analyze and validate ([d3ac535](https://github.com/madebydaniz/linguasteg/commit/d3ac53566a785b9c0c94c729aae56404b1f0e78b))
* **cli:** add usable encode/decode args with file and format support ([1ebba25](https://github.com/madebydaniz/linguasteg/commit/1ebba257be0f188a16f902b50b45ece2a5e995c1))
* **cli:** add validate command for trace integrity checks ([6b098b7](https://github.com/madebydaniz/linguasteg/commit/6b098b7b454f591949dd88127c51db74eb07085f))
* **cli:** add zero-url starter datasets and encode dataset guidance notice ([77da95f](https://github.com/madebydaniz/linguasteg/commit/77da95f4ae3529b38a4520db943b448bf2a31ab5))
* **cli:** apply installed lexicon dataset variants during encode ([6443ba2](https://github.com/madebydaniz/linguasteg/commit/6443ba2c6867273f75441398cb1a1b0a17b95ad9))
* **cli:** auto-activate default datasets and add source list discovery ([37c1a3c](https://github.com/madebydaniz/linguasteg/commit/37c1a3c87604700adc313f426a252dc0c310f244))
* **cli:** enforce strict proto-encode trace json contract validation ([2f704d1](https://github.com/madebydaniz/linguasteg/commit/2f704d1b5e14c7d8d289bb9235fc0fea85ac1af6))
* **cli:** enforce trace-only lossless decode and align tests with emit-trace ([9524d27](https://github.com/madebydaniz/linguasteg/commit/9524d276b2c06b9187d2fdc39a22cd01615d7351))
* **cli:** extend catalog with templates profiles and language filter ([e7e0a3f](https://github.com/madebydaniz/linguasteg/commit/e7e0a3f97190cbe9c2fac02e26055edb2da966f0))
* **cli:** honor --data-dir for decode/analyze/validate text normalization ([bc1c73f](https://github.com/madebydaniz/linguasteg/commit/bc1c73f747de6d88b24a9314fb4a8bb04fb20eef))
* **cli:** implement analyze command with trace integrity metrics ([cd24d77](https://github.com/madebydaniz/linguasteg/commit/cd24d77f60bc258590092184d99eb35834d081a0))
* **cli:** include schemas section in catalog outputs ([f133c9c](https://github.com/madebydaniz/linguasteg/commit/f133c9c49ba8d649508b9ab4fa7aa7fdc9a76899))
* **cli:** make first english frame secret-sensitive via intro surface variants ([f8c84d9](https://github.com/madebydaniz/linguasteg/commit/f8c84d97c11848142c1f194489ebea3b13a43a6b))
* **cli:** normalize dataset variants during text-input decode and analyze ([30b8e68](https://github.com/madebydaniz/linguasteg/commit/30b8e68cfd903a6d29f59b8b3387ca90d74d3422))
* **cli:** require secrets for encode/decode and add optional secret-aware analyze ([66e07fd](https://github.com/madebydaniz/linguasteg/commit/66e07fd186b3722b0f6c4a67ae80083a52cec99e))
* **cli:** support data artifact-url install with manifest hash tracking ([1f0c1a9](https://github.com/madebydaniz/linguasteg/commit/1f0c1a919f375ca00c90cb2f99dc22a5473d0f65))
* **cli:** support decoding proto-encode JSON output in proto-decode ([9a300db](https://github.com/madebydaniz/linguasteg/commit/9a300db66952ee2c74e8b7e12898bd7ff4bd6940))
* **cli:** validate lexicon dataset artifacts during data install ([ad1c198](https://github.com/madebydaniz/linguasteg/commit/ad1c1989a1f2ac408df0317f61be67b53dcf9200))
* **cli:** wire encode profile selection through runtime and farsi realization ([cf6b270](https://github.com/madebydaniz/linguasteg/commit/cf6b270c2294bc3121fe31c39729d47517ff53a2))
* **cli:** wire encode to installed dataset source selection ([f0eab02](https://github.com/madebydaniz/linguasteg/commit/f0eab02061e838b9f84842af747095f121cb6d04))
* **cli:** wire italian runtime, data source, and trace detection ([e8fc952](https://github.com/madebydaniz/linguasteg/commit/e8fc9526b3c553a3c0970bb2f9b3718b42584ed3))
* **cli:** wire proto encode decode commands to pipeline orchestrator ([14c4cd8](https://github.com/madebydaniz/linguasteg/commit/14c4cd8fc1281cda44986be6458fd02cb48f8c3e))
* **core:** add text extractor contract and runtime wiring ([9c24f9c](https://github.com/madebydaniz/linguasteg/commit/9c24f9c6f3e69272249a4cc05f3cc1e440bec36d))
* **en:** add author-inspired style profiles with reversible lexical variants ([35fbcaa](https://github.com/madebydaniz/linguasteg/commit/35fbcaa64ad1c5b626d736a68213e07648527039))
* **en:** enable lossless plain-text decode via extractor and reverse mapping ([e83c1e4](https://github.com/madebydaniz/linguasteg/commit/e83c1e43f59bf2c88ce248ee9678badf56127bbc))
* **fa:** enable lossless plain-text decode with full-width symbolic inventories ([0d11dd9](https://github.com/madebydaniz/linguasteg/commit/0d11dd951e14cd426562a829779987500b2fd282))
* **it:** localize italian prototype lexicon, parser, and starter dataset ([2ad6c52](https://github.com/madebydaniz/linguasteg/commit/2ad6c5268b2e85cca085e2cdf13476941c2ce711))
* **models:** scaffold german language pack and runtime registration ([4935f52](https://github.com/madebydaniz/linguasteg/commit/4935f520c3adcca6ae52e651e4946f47e1338189))


### 🐛 Bug Fixes

* **cargo:** stabilize workspace internal crate versioning for release please ([e11fc8c](https://github.com/madebydaniz/linguasteg/commit/e11fc8c46b66fa8dd25c51e268551c5a557b153f))
* **cargo:** stabilize workspace internal crate versioning for release-please ([384ae31](https://github.com/madebydaniz/linguasteg/commit/384ae3141a76be8a231b8a4e9a4f6cf14909141a))
* **cli:** allow terminal partial proto-encode frames and add contract compatibility matrix tests ([0e79c36](https://github.com/madebydaniz/linguasteg/commit/0e79c36eb71002b99c1bcd986d97068b8fb6e9f0))
* **cli:** auto-detect english plain-text input in decode analyze validate ([8439466](https://github.com/madebydaniz/linguasteg/commit/84394662493622f4c5ffccb39e292842c16eebea))
* **cli:** harden decode errors for non-envelope and invalid envelope payloads ([353dc7c](https://github.com/madebydaniz/linguasteg/commit/353dc7cea59c0bc90b3fe3ad384264e1f1d8e10f))
* **cli:** stabilize secret symbolic mixing with decode/analyze compatibility ([4227c2b](https://github.com/madebydaniz/linguasteg/commit/4227c2ba15fd62bf848d171fb1cb6ef16dbc39c7))
* **cli:** validate trace frame sequence before decode and integrity analysis ([4af21db](https://github.com/madebydaniz/linguasteg/commit/4af21db1b5e2b2a57e12ae8e30fa398c98037299))
* **cli:** write command output via stdout stream to avoid codeql cleartext-log false positive ([dfce7ff](https://github.com/madebydaniz/linguasteg/commit/dfce7ff134978d03c9544ef6fc62a73b931152fe))
* **core,cli:** address review findings for dataset normalization, json escaping, and padding guards ([3d79218](https://github.com/madebydaniz/linguasteg/commit/3d7921872964cc0193c78b589f413029fab5db95))
* **release:** use explicit crate versions for release-please cargo-workspace compatibility ([2307fb4](https://github.com/madebydaniz/linguasteg/commit/2307fb442da92ea6a2ad015aa1b75befba018a6b))


### 🧱 Refactoring

* **cli:** accept generic language codes for proto legacy commands ([2ac5fed](https://github.com/madebydaniz/linguasteg/commit/2ac5fedfeb1e0aed8e6ccaede5e5983f26b3ee33))
* **cli:** accept generic language codes in parsing and target routing ([535a12f](https://github.com/madebydaniz/linguasteg/commit/535a12f5e436337157be66e2f2829071af4a23e6))
* **cli:** centralize lang and format flag parsing helpers ([2ffa10d](https://github.com/madebydaniz/linguasteg/commit/2ffa10df2118a64e870aee65e018d9e33ade16b8))
* **cli:** centralize runtime init and unsupported-language diagnostics ([6e6446f](https://github.com/madebydaniz/linguasteg/commit/6e6446ff0dcc67c9d78b96d3e28b40c6748012a6))
* **cli:** centralize secret flag parsing across commands ([638a2b3](https://github.com/madebydaniz/linguasteg/commit/638a2b3717feaab025598c28a47ef8f10633ab13))
* **cli:** centralize trace language resolution and reject mixed-language traces ([e6c77e5](https://github.com/madebydaniz/linguasteg/commit/e6c77e507ccae8d54bf87963da4723e88039883f))
* **cli:** consolidate runtime target iteration in discovery collectors ([8cfff35](https://github.com/madebydaniz/linguasteg/commit/8cfff352fc5ee7c368ad3a64aa8783898b3aa99a))
* **cli:** derive supported model languages from runtime providers ([62658e3](https://github.com/madebydaniz/linguasteg/commit/62658e397cfc4c3ebe29a63c489701fa7649587d))
* **cli:** extract encode payload source argument parser ([463b6d4](https://github.com/madebydaniz/linguasteg/commit/463b6d47384aa6a1cf57ee454ce97de0576fcec7))
* **cli:** harden trace parsing with serde_json-backed proto-encode decoding ([9c220fe](https://github.com/madebydaniz/linguasteg/commit/9c220fea75a2b4c8c54342a67886e980f4f8d87c))
* **cli:** introduce language-code runtime provider registry ([010ee94](https://github.com/madebydaniz/linguasteg/commit/010ee94458dc91ce67d27b8b3dd257752f4627e3))
* **cli:** load data sources from manifest and add source override for install/update ([45205a1](https://github.com/madebydaniz/linguasteg/commit/45205a14a73557e6ae25e5264257ae60e354fc3e))
* **cli:** migrate prototype runtime to trait-object architecture for language extensibility ([d26442e](https://github.com/madebydaniz/linguasteg/commit/d26442e9b3fc3d04cba89f0cec21b1993be8ab22))
* **cli:** remove gateway stub output from encode/decode reports ([948e75c](https://github.com/madebydaniz/linguasteg/commit/948e75cce829c7201703088e7a0b1bd7ab96e901))
* **cli:** route decode parsing through shared trace args parser ([1c757a2](https://github.com/madebydaniz/linguasteg/commit/1c757a28db910744e58f4fe6f23bc537e15ac494))
* **cli:** split main into args commands formatters and analysis modules ([ac7f461](https://github.com/madebydaniz/linguasteg/commit/ac7f461dbf70260e503b0bfc76c7adbf4fe62dad))
* **cli:** standardize command parsing and exit codes ([71409ab](https://github.com/madebydaniz/linguasteg/commit/71409ab08244332c69031685058303097c9a31e4))
* **cli:** standardize input-mode error taxonomy across decode analyze validate ([ecd55d7](https://github.com/madebydaniz/linguasteg/commit/ecd55d7e273ae7e12bd39e72f376ab0b91cb3173))
* **cli:** standardize security error contract for secret encryption and decryption flows ([3b90b85](https://github.com/madebydaniz/linguasteg/commit/3b90b850745aa200cabf32ef3617dca5e38cf99a))
* **cli:** standardize usage error output with global error contract ([d4f15f7](https://github.com/madebydaniz/linguasteg/commit/d4f15f7e35646c243b56460020d2a5be255bc7fb))
* **cli:** unify analyze and validate trace-arg parsing ([6c78ac7](https://github.com/madebydaniz/linguasteg/commit/6c78ac7e7f9a7adbd4217082899758294b240cf7))
* **cli:** unify discovery command argument parsing ([14bd3c8](https://github.com/madebydaniz/linguasteg/commit/14bd3c845fb273b457a886f34ab7473696e8798b))
* **cli:** unify proto trace contract parsing across trace and language resolution ([bff3a67](https://github.com/madebydaniz/linguasteg/commit/bff3a6747af7702803823f2ece2eba165dce95f8))
* **cli:** unify structured error model with stable codes and categories ([81d59ef](https://github.com/madebydaniz/linguasteg/commit/81d59ef7f044d46f5c56de4287c231bfe4ec6ffe))


### ✅ Tests

* **cli:** add de/it roundtrip and italian file I/O integration coverage ([a3cebe9](https://github.com/madebydaniz/linguasteg/commit/a3cebe90f8c4f3f3e8299e1f953aa39d73aa984d))
* **cli:** add english golden fixtures for decode and analyze contracts ([bdc34fe](https://github.com/madebydaniz/linguasteg/commit/bdc34febe92875e5dc49237adef6da4dad3bfa4b))
* **cli:** add golden fixtures for catalog templates profiles and validate contracts ([83d8a97](https://github.com/madebydaniz/linguasteg/commit/83d8a97964b5cd2d8203ee5393f2e68029287d14))
* **cli:** add golden fixtures for schemas command contracts ([5a026b2](https://github.com/madebydaniz/linguasteg/commit/5a026b26e85d5942a37c5722070011cbb4ca4448))
* **cli:** add golden fixtures for unicode, whitespace, and invalid trace edge cases ([5e5ee71](https://github.com/madebydaniz/linguasteg/commit/5e5ee7116ad57a8988cb67d1fb9903f898441942))
* **cli:** add golden output fixtures and deterministic env-isolated CLI tests ([9a51d2b](https://github.com/madebydaniz/linguasteg/commit/9a51d2bb7c29d89bdf2d800fcc8484dbb7f46526))
* **cli:** add integration coverage for encode decode analyze flows ([a253bd1](https://github.com/madebydaniz/linguasteg/commit/a253bd18d995df87327e245db61825641b48de91))
* **cli:** add unit coverage for argument parsing helpers ([becab7a](https://github.com/madebydaniz/linguasteg/commit/becab7a9128bfcd443f0cc92793b95ff39518920))
* **cli:** add unit coverage for trace parser line and json paths ([8d737eb](https://github.com/madebydaniz/linguasteg/commit/8d737eb32d2445dbfd1510508e85c8d5bc382abc))
* **cli:** cover secret-file flow and CLI-over-env secret precedence ([3c2cab1](https://github.com/madebydaniz/linguasteg/commit/3c2cab106d272fada89d02c34b1e031943d8127d))


### 👽 Miscellaneous

* **ci:** upgrade codeql workflow to v4 and sanitize domain errors ([ad6546c](https://github.com/madebydaniz/linguasteg/commit/ad6546c2f7bd415b07cae60196df4a61edadf094))
* **workspace:** bootstrap virtual workspace skeleton with facade crate ([5222c59](https://github.com/madebydaniz/linguasteg/commit/5222c590e0220b0e486dd4b1800356a577017657))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * linguasteg-core bumped from 0.1.0 to 0.2.0
    * linguasteg-models bumped from 0.1.0 to 0.2.0
