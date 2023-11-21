use near_lake_context_derive::LakeContext;
use near_lake_framework::LakeBuilder;
use tracing_subscriber::EnvFilter;

#[derive(LakeContext)]
struct Context {}

async fn handle_block(
    _block: near_lake_primitives::block::Block,
    _ctx: &Context,
) -> anyhow::Result<()> {
    println!("new block");
    Ok(())
}

// Steps to follow:
// 1) docker run -p 4566:4566 -e SERVICES=s3 localstack/localstack:3.0.0
// 2) cargo test daniyar
#[test]
fn daniyar() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("near_lake_framework=debug".parse()?),
        );
    subscriber.init();
    let mut lake_builder = LakeBuilder::default()
        .s3_bucket_name("near-lake-custom")
        .s3_region_name("us-east-1")
        .start_block_height(0);
    let lake = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let aws_config = aws_config::from_env().load().await;
            let s3_config =
                aws_sdk_s3::config::Builder::from(&aws_types::SdkConfig::from(aws_config))
                    .endpoint_url("http://[::1]:4566")
                    .build();
            lake_builder = lake_builder.s3_config(s3_config);
            lake_builder.build()
        })?;
    lake.run_with_context(handle_block, &Context {})?;

    Ok(())
}
