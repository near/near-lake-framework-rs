# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/near/near-jsonrpc-client-rs/compare/v0.3.0...HEAD)

## [0.3.0](https://github.com/near/near-lake-framework/compare/v0.2.0...v0.3.0) - 2022-06-10

- Introduce `LakeConfigBuilder` for creating configs
  ```rust
  let config = LakeConfigBuilder.default()
    .testnet()
    .start_block_height(88220926)
    .build()
    .expect("Failed to build LakeConfig");
  ```
- Now you can provide custom AWS SDK S3 `Config`
  ```rust
  use aws_sdk_s3::Endpoint;
  use http::Uri;
  use near_lake_framework::LakeConfigBuilder;

  let aws_config = aws_config::from_env().load().await;
  let mut s3_conf = aws_sdk_s3::config::Builder::from(&aws_config);
  s3_conf = s3_conf
    .endpoint_resolver(
      Endpoint::immutable("http://0.0.0.0:9000".parse::<Uri>().unwrap()))
    .build();

  let config = LakeConfigBuilder::default()
    .s3_config(s3_conf)
    .s3_bucket_name("near-lake-data-custom")
    .start_block_height(1)
    .build()
    .expect("Failed to build LakeConfig");
  ```

### Breaking change

`LakeConfig` has a breaking change as we've removed `s3_endpoint` and added `s3_config`. Please, consider migrating to use `LakeConfigBuilder` instead of directly crafting the `Lakeconfig`

[0.3.0]: https://github.com/near/near-lake-framework/releases/tag/v0.3.0

## [0.2.0] - 2022-04-25

The first public release. See [announcement on NEAR Gov Forum](https://gov.near.org/t/announcement-near-lake-framework-brand-new-word-in-indexer-building-approach/17668)

> Release Page: <https://github.com/near/near-lake-framework/releases/tag/v0.2.0>

[0.2.0]: https://github.com/near/near-lake-framework/releases/tag/v0.2.0
