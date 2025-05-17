use aws_config::{SdkConfig, meta::region::RegionProviderChain};

use aws_sdk_ec2::config::Region;
use aws_sdk_ssm::Client as SsmClient;

pub(super) async fn get_config(
    region: &Option<String>,
) -> Result<SdkConfig, Box<dyn std::error::Error>> {
    let region_provider = match region {
        Some(r) => RegionProviderChain::first_try(Region::new(r.clone())).or_default_provider(),
        None => RegionProviderChain::default_provider(),
    };
    let config = aws_config::from_env().region(region_provider).load().await;
    Ok(config)
}

pub(super) async fn get_al2023_ami(
    ssm_client: &SsmClient,
) -> Result<String, Box<dyn std::error::Error>> {
    // AL2023 x86_64 SSM parameter name
    let param_name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";

    // Fetch the parameter value (AMI ID)
    let result = ssm_client.get_parameter().name(param_name).send().await?;

    let ami_id = result
        .parameter()
        .and_then(|p| p.value())
        .ok_or("AMI ID not found in parameter store")?
        .to_string();

    Ok(ami_id)
}
