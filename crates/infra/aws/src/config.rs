use aws_config::{SdkConfig, meta::region::RegionProviderChain};

use aws_sdk_ec2::config::Region;
use aws_sdk_ssm::Client as SsmClient;
use aws_sdk_ssm::error::SdkError;
use aws_sdk_ssm::operation::get_parameter::GetParameterError;

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

// pub(super) async fn save_subnet_to_ssm(
//     ssm: &SsmClient,
//     az: &str,
//     subnet_id: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let key = format!("/byocvpn/subnets/{az}");
//     ssm.put_parameter()
//         .r#type(aws_sdk_ssm::types::ParameterType::String)
//         .name(key)
//         .value(subnet_id)
//         .overwrite(true)
//         .send()
//         .await?;

//     Ok(())
// }

// pub async fn get_subnet_id(
//     ssm: &SsmClient,
//     az: &str,
// ) -> Result<Option<String>, Box<dyn std::error::Error>> {
//     let key = format!("/byocvpn/subnets/{az}");
//     match ssm.get_parameter().name(&key).send().await {
//         Ok(resp) => Ok(resp
//             .parameter()
//             .and_then(|p| p.value())
//             .map(|v| v.to_string())),
//         Err(SdkError::ServiceError(err)) => match err.err() {
//             GetParameterError::ParameterNotFound(_) => Ok(None),
//             _ => Err(Box::new(SdkError::ServiceError(err))),
//         },

//         Err(e) => Err(Box::new(e)),
//     }
// }
