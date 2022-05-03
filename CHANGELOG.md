# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Introduce `LakeConfigBuilder` for creating configs
- Now you can provide AWS credentials through the `LakeConfig`
  ```rust
  let config = LakeConfigBuilder::default()
    .s3_bucket_name("near-lake-data-testnet")
    .start_block_height(88220926)
    .with_custom_credentials(
        "AKIAIOSFODNN7EXAMPLE",
        "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    )
    .build()
    .expect("Filed to build LakeConfig");
  ```
- New example (`printer`) is added to `examples` folder

### Breaking change

`LakeConfig` has a breaking change as we've added an optional field `s3_credentials`. Please, consider migrating to use `LakeConfigBuilder` instead of directly crafting the `Lakeconfig`

## [0.2.0] - 2022-04-25

The first public release. See [announcement on NEAR Gov Forum](https://gov.near.org/t/announcement-near-lake-framework-brand-new-word-in-indexer-building-approach/17668)

> Release Page: <https://github.com/near/near-lake-framework/releases/tag/v0.2.0>

[0.2.0]: https://github.com/near/near-lake-framework/releases/tag/v0.2.0
